# `inferno`
`inferno` is a wiki-management system written in
[Rust](https://www.rust-lang.org/) and held up by the aching back of
[`leptos`](https://leptos.dev/).

Inferno subscribes to a minimalist philosophy that wikis should only be the
requested content and nothing else. Lightweight installations, opinionated
design to get writing wikis more quickly, and *zero* JavaScript bloat.

In fact, Inferno is completely usable without JavaScript.
[Graceful Degradation](https://developer.mozilla.org/en-US/docs/Glossary/Graceful_degradation)
is a key concept in both `leptos` and Inferno, and Inferno is battle-tested in
some of the most extreme browsing environments (most notably
[`lynx`](https://lynx.invisible-island.net/), because if the website can
support `lynx` cleanly it can support **anything**).

Check out a running instance at <https://infernowiki.rs/>, but be gentle,
please.

## `#![forbid(unsafe_code)]`
Unsafe code is forbidden. Classically and interestingly, many of the big day-0
exploits happen because of simple oversights that would otherwise be
bounds-checked by the compiler or at runtime.

I expect `inferno` to grow along with the Rust ecosystem, which means I can
expect to be pulling crates to do jobs that might be done with `unsafe` code
that are battle-tested.
