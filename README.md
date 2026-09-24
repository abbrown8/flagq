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

flag new-checkout {
    default: off
    rule: on if plan = "pro" or plan = "team"
    rule: on if country = "us" and beta != "false"
}
```

Each flag has a default and an ordered list of rules. Rules are checked
top to bottom; the first one whose conditions all match wins. If nothing
matches, the flag falls back to its default. Values are compared as plain
strings, so `beta = "true"` matches a context where you pass `beta=true`.

A rule's conditions can be joined with `and` and `or`, and `and` binds
tighter than `or`, the same as in most languages: `a = "1" and b = "2" or
c = "3"` means `(a = "1" and b = "2") or c = "3"`. Conditions can also be
negated with `!=`, which matches when the key is absent from the context
or holds a different value.

A condition can also compare numerically with `<`, `<=`, `>`, or `>=`,
for things like account age or a usage count:

```
flag high-volume {
    default: off
    rule: on if requests_per_day >= 1000
}
```

The right-hand side of a numeric comparison has to parse as a number;
`flagq` rejects the flags file at parse time if it doesn't. The
left-hand side comes from the context passed on the command line, so if
that value isn't a number (or the key is missing entirely) the condition
just doesn't match, the same as a `=` condition on a missing key.

A condition can also gate on a percentage rollout with `rollout`:

```
flag beta-analytics {
    default: off
    rule: on if user_id rollout 25%
}
```

This turns on for roughly 25% of the values seen for `user_id`. The
bucket a given value falls into is stable for a given flag: the same
`user_id` always lands in the same bucket for `beta-analytics`, but a
different flag rolling out on `user_id` gets an independent split. The
percentage has to be a number between 0 and 100, checked at parse time.

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

Early. The grammar covers equality, inequality, numeric comparisons, and
percentage rollouts, joined with `and` / `or`.
