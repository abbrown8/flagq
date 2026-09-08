# flagq

A command-line tool that answers one question: is this feature flag on for
this specific context, and why.

Most feature-flag systems keep their rules in a dashboard, which is fine
until you need to know exactly why a flag flipped for one user last
Tuesday, or you're reviewing a pull request that touches a flags file and
want to check the change before it merges. `flagq` reads a small text file
of flag definitions and evaluates one flag against a set of key=value
context values, on the command line, with no server involved.

## The flags file format

```
flag new-checkout {
    default: off
    rule: on if plan = "pro"
    rule: on if country = "us" and beta = "true"
}

flag dark-mode {
    default: on
    rule: off if legacy_browser = "true"
}
```

Each flag has a default and an ordered list of rules. Rules are checked
top to bottom; the first one whose conditions all match wins. If nothing
matches, the flag falls back to its default. Values are compared as plain
strings, so `beta = "true"` matches a context where you pass `beta=true`.

## Usage

```
$ flagq flags.txt new-checkout plan=pro
new-checkout: on
reason: rule 1 matched (plan = "pro")

$ flagq flags.txt new-checkout country=us beta=true
new-checkout: on
reason: rule 2 matched (country = "us" and beta = "true")

$ flagq flags.txt new-checkout plan=free
new-checkout: off
reason: no rule matched, fell through to default
```

## Errors that actually point at the problem

If the flags file has a mistake, `flagq` reports it the way a compiler
would: the exact line and column, the source line itself, and a caret
under the offending token.

```
$ flagq flags.txt new-checkout plan=pro
error: expected `{` after flag name, found end of file
 --> flags.txt:1:19
  |
1 | flag new-checkout
  |                   ^
```

Typos in the flag name or in a context key get a "did you mean" suggestion
instead of a bare failure:

```
$ flagq flags.txt new-checkot plan=pro
error: no flag named `new-checkot` in flags.txt
  did you mean `new-checkout`?
  defined flags: new-checkout, dark-mode
```

## Building

Standard library only, no external crates:

```
cargo build --release
```

## Status

Early. The grammar covers equality conditions ANDed together, which
handles the common cases but not `or`, negation, or numeric comparisons
yet.
