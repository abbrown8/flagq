mod error;
mod lexer;
mod parser;

use error::suggest;
use lexer::Lexer;
use parser::{Flag, Parser, Rule};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::ExitCode;

fn usage() -> String {
    "usage: flagq <flags-file> <flag-name> [key=value ...]\n\n\
     example:\n  \
     flagq flags.txt new-checkout plan=pro country=us\n"
        .to_string()
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        eprint!("{}", usage());
        return ExitCode::FAILURE;
    }

    let path = &args[0];
    let flag_name = &args[1];
    let mut context: HashMap<String, String> = HashMap::new();
    for arg in &args[2..] {
        match arg.split_once('=') {
            Some((k, v)) => {
                context.insert(k.to_string(), v.to_string());
            }
            None => {
                eprintln!("error: expected key=value, found `{}`", arg);
                return ExitCode::FAILURE;
            }
        }
    }

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read `{}`: {}", path, e);
            return ExitCode::FAILURE;
        }
    };

    let tokens = match Lexer::new(&source).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprint!("{}", e.render(path, &source));
            return ExitCode::FAILURE;
        }
    };

    let flags = match Parser::new(tokens).parse_file() {
        Ok(f) => f,
        Err(e) => {
            eprint!("{}", e.render(path, &source));
            return ExitCode::FAILURE;
        }
    };

    let Some(flag) = flags.iter().find(|f| &f.name == flag_name) else {
        eprintln!("error: no flag named `{}` in {}", flag_name, path);
        if let Some(near) = suggest(flag_name, flags.iter().map(|f| f.name.as_str())) {
            eprintln!("  did you mean `{}`?", near);
        }
        if !flags.is_empty() {
            eprintln!(
                "  defined flags: {}",
                flags
                    .iter()
                    .map(|f| f.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        return ExitCode::FAILURE;
    };

    warn_on_unknown_keys(flag, &context);

    let (result, reason) = evaluate(flag, &context);
    println!("{}: {}", flag.name, if result { "on" } else { "off" });
    println!("reason: {}", reason);

    ExitCode::SUCCESS
}

fn warn_on_unknown_keys(flag: &Flag, context: &HashMap<String, String>) {
    let known: Vec<&str> = flag
        .rules
        .iter()
        .flat_map(|r| r.conditions.iter().map(|c| c.key.as_str()))
        .collect();
    for key in context.keys() {
        if !known.contains(&key.as_str()) {
            if let Some(near) = suggest(key, known.iter().copied()) {
                eprintln!(
                    "warning: `{}` is not used by any rule on `{}` (did you mean `{}`?)",
                    key, flag.name, near
                );
            }
        }
    }
}

/// Evaluates the rules in order and returns both the outcome and a plain
/// explanation of why that outcome was reached, so the answer to "is this
/// flag on" always comes with the "because" attached.
fn evaluate(flag: &Flag, context: &HashMap<String, String>) -> (bool, String) {
    for (i, rule) in flag.rules.iter().enumerate() {
        if rule_matches(rule, context) {
            let clause = describe_conditions(rule);
            return (
                rule.effect,
                format!("rule {} matched ({})", i + 1, clause),
            );
        }
    }
    (
        flag.default,
        "no rule matched, fell through to default".to_string(),
    )
}

fn rule_matches(rule: &Rule, context: &HashMap<String, String>) -> bool {
    rule.conditions.iter().all(|cond| {
        context
            .get(&cond.key)
            .map(|v| v == &cond.value)
            .unwrap_or(false)
    })
}

fn describe_conditions(rule: &Rule) -> String {
    rule.conditions
        .iter()
        .map(|c| format!("{} = \"{}\"", c.key, c.value))
        .collect::<Vec<_>>()
        .join(" and ")
}
