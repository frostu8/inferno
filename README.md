# `inferno`
`inferno` is a wiki-management system written in
[Rust](https://www.rust-lang.org/).

Inferno subscribes to a minimalist philosophy that wikis should only be the
requested content and nothing else.

Check out a running instance at <https://infernowiki.rs/>, but be gentle,
please.

## `#![forbid(unsafe_code)]`
Unsafe code is forbidden. Classically and interestingly, many of the big day-0
exploits happen because of simple oversights that would otherwise be
bounds-checked by the compiler or at runtime.

I expect `inferno` to grow along with the Rust ecosystem, which means I can
expect to be pulling crates to do jobs that might be done with `unsafe` code
that are battle-tested.
