*Part 3/10*
📰 **UPDATES FROM THE RUST PROJECT** 📰
448 pull requests were [merged in the last week](https://github.com/search?q=is%3Apr+org%3Arust-lang+is%3Amerged+merged%3A2025-06-17..2025-06-24)
**Compiler**
• [perf: Cache the canonical instantiation of param\-envs](https://github.com/rust-lang/rust/pull/142316)
• [asyncDrop trait without sync Drop generates an error](https://github.com/rust-lang/rust/pull/142606)
• [stabilize generic\_arg\_infer](https://github.com/rust-lang/rust/pull/141610)
• [skip no\-op drop glue](https://github.com/rust-lang/rust/pull/142508)
**Library**
• [add trim\_prefix and trim\_suffix methods for both slice and str types](https://github.com/rust-lang/rust/pull/142331)
• [allow comparisons between CStr, CString, and Cow<CStr\>](https://github.com/rust-lang/rust/pull/137268)
• [allow storing format\_args\!\(\) in variable](https://github.com/rust-lang/rust/pull/140748)
• [impl Default for array::IntoIter](https://github.com/rust-lang/rust/pull/141574)
• [change core::iter::Fuse's Default impl to do what its docs say it does](https://github.com/rust-lang/rust/pull/140985)
• [let String pass \#\[track\_caller\] to its Vec calls](https://github.com/rust-lang/rust/pull/142728)
• [safer implementation of RepeatN](https://github.com/rust-lang/rust/pull/130887)
• [use a distinct ToString implementation for u128 and i128](https://github.com/rust-lang/rust/pull/142294)
**Cargo**
• [cargo: feat\(toml\): Parse support for multiple build scripts](https://github.com/rust-lang/cargo/pull/15630)
• [cargo: feat: introduce perma unstable \-\-compile\-time\-deps option for cargo build](https://github.com/rust-lang/cargo/pull/15674)
• [cargo: fix potential deadlock in CacheState::lock](https://github.com/rust-lang/cargo/pull/15698)
**Rustdoc**
• [avoid a few more allocations in write\_shared\.rs](https://github.com/rust-lang/rust/pull/142667)
• [rustdoc\-json: keep empty generic args if parenthesized](https://github.com/rust-lang/rust/pull/142932)
• [rustdoc: make srcIndex no longer a global variable](https://github.com/rust-lang/rust/pull/142100)
**Clippy**
• [use jemalloc for Clippy](https://github.com/rust-lang/rust/pull/142286)
• [perf: Don't spawn so many compilers \(3/2\) \(19m → 250k\)](https://github.com/rust-lang/rust-clippy/pull/15030)
• [Sugg: do not parenthesize a double unary operator](https://github.com/rust-lang/rust-clippy/pull/14983)
• [or\_fun\_call: lint more methods](https://github.com/rust-lang/rust-clippy/pull/15071)
• [add missing space when expanding a struct\-like variant](https://github.com/rust-lang/rust-clippy/pull/15096)
• [check MSRV before suggesting applying const to a function](https://github.com/rust-lang/rust-clippy/pull/15080)
• [emit lint about redundant closure on the closure node itself](https://github.com/rust-lang/rust-clippy/pull/14791)
• [fix branches\_sharing\_code suggests misleadingly when in assignment](https://github.com/rust-lang/rust-clippy/pull/15076)
• [fix clippy::question\_mark on let\-else with cfg](https://github.com/rust-lang/rust-clippy/pull/15082)
• [fix exhaustive\_structs false positive on structs with default valued field](https://github.com/rust-lang/rust-clippy/pull/15022)
• [fix manual\_ok\_err suggests wrongly with references](https://github.com/rust-lang/rust-clippy/pull/15053)
• [fix non\_copy\_const ICE](https://github.com/rust-lang/rust-clippy/pull/15083)
• [fix wildcard\_enum\_match\_arm suggests wrongly with raw identifiers](https://github.com/rust-lang/rust-clippy/pull/15093)
• [fix false positive of borrow\_deref\_ref](https://github.com/rust-lang/rust-clippy/pull/14967)
• [fix suggestion\-causes\-error of empty\_line\_after\_outer\_attr](https://github.com/rust-lang/rust-clippy/pull/15078)
• [new lint: manual\_is\_multiple\_of](https://github.com/rust-lang/rust-clippy/pull/14292)
**Rust\-Analyzer**
