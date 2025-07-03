*Часть 3/5*
📰 **UPDATES FROM THE RUST PROJECT**
• 448 pull requests were [merged in the last week](https://github.com/search?q=is%3Apr+org%3Arust-lang+is%3Amerged+merged%3A2025-06-17..2025-06-24)Compiler
• [perf: Cache the canonical instantiation of param\-envs](https://github.com/rust-lang/rust/pull/142316)
• [asyncDrop trait without sync Drop generates an error](https://github.com/rust-lang/rust/pull/142606)
• [stabilize generic\_arg\_infer](https://github.com/rust-lang/rust/pull/141610)
• [skip no\-op drop glue](https://github.com/rust-lang/rust/pull/142508)
• Library
• [add trim\_prefix and trim\_suffix methods for both slice and str types](https://github.com/rust-lang/rust/pull/142331)
• [allow comparisons between CStr, CString, and Cow<CStr\>](https://github.com/rust-lang/rust/pull/137268)
• [allow storing format\_args\!\(\) in variable](https://github.com/rust-lang/rust/pull/140748)
• [impl Default for array::IntoIter](https://github.com/rust-lang/rust/pull/141574)
• [change core::iter::Fuse's Default impl to do what its docs say it does](https://github.com/rust-lang/rust/pull/140985)
• [let String pass \#\[track\_caller\] to its Vec calls](https://github.com/rust-lang/rust/pull/142728)
• [safer implementation of RepeatN](https://github.com/rust-lang/rust/pull/130887)
• [use a distinct ToString implementation for u128 and i128](https://github.com/rust-lang/rust/pull/142294)
• Cargo
• [cargo: feat\(toml\): Parse support for multiple build scripts](https://github.com/rust-lang/cargo/pull/15630)
• [cargo: feat: introduce perma unstable \-\-compile\-time\-deps option for cargo build](https://github.com/rust-lang/cargo/pull/15674)
• [cargo: fix potential deadlock in CacheState::lock](https://github.com/rust-lang/cargo/pull/15698)
• Rustdoc
• [avoid a few more allocations in write\_shared\.rs](https://github.com/rust-lang/rust/pull/142667)
• [rustdoc\-json: keep empty generic args if parenthesized](https://github.com/rust-lang/rust/pull/142932)
• [rustdoc: make srcIndex no longer a global variable](https://github.com/rust-lang/rust/pull/142100)
• Clippy
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
• Rust\-Analyzer
• [rust\-analyzer: add fn parent\(self, db\) → GenericDef to hir::TypeParam](https://github.com/rust-lang/rust-analyzer/pull/20046)
• [rust\-analyzer: cleanup folding\_ranges and support more things](https://github.com/rust-lang/rust-analyzer/pull/20080)
• [rust\-analyzer: do not default to 'static for trait object lifetimes](https://github.com/rust-lang/rust-analyzer/pull/20036)
• [rust\-analyzer: closure capturing for let exprs](https://github.com/rust-lang/rust-analyzer/pull/20039)
• [rust\-analyzer: fix cargo project manifest not pointing to the workspace root](https://github.com/rust-lang/rust-analyzer/pull/20069)
• [rust\-analyzer: in "Wrap return type" assist, don't wrap exit points if they already have the right type](https://github.com/rust-lang/rust-analyzer/pull/20061)
• [rust\-analyzer: respect \.cargo/config\.toml build\.target\-dir](https://github.com/rust-lang/rust-analyzer/pull/20072)
• [rust\-analyzer: temporarily disable \+ typing handler as it moves the cursor position](https://github.com/rust-lang/rust-analyzer/pull/20042)
• [rust\-analyzer: use ROOT hygiene for args inside new format\_args\! expansion](https://github.com/rust-lang/rust-analyzer/pull/20073)
• [rust\-analyzer: hide imported privates if private editable is disabled](https://github.com/rust-lang/rust-analyzer/pull/20025)
• [rust\-analyzer: mimic rustc's new format\_args\! expansion](https://github.com/rust-lang/rust-analyzer/pull/20056)
Rust Compiler Performance TriageA week dominated by the landing of a large patch implementing [RFC\#3729](https://github.com/rust-lang/rfcs/pull/3729) which unfortunately introduced rather sizeable performance regressions \(avg of \~1% instruction count on 111 primary benchmarks\)\. This was deemed worth it so that the patch could land and performance could be won back in follow up PRs\.Triage done by @rylev\. Revision range: [45acf54e\.\.42245d34](https://perf.rust-lang.org/?start=45acf54eea118ed27927282b5e0bfdcd80b7987c&end=42245d34d22ade32b3f276dcf74deb826841594c&absolute=false&stat=instructions%3Au)Summary:
| \(instructions:u\)              | mean    | range                 | count |
| Regressions ❌  \(primary\)      | 1\.1%   | \[0\.2%, 9\.1%\]      | 123   |
| Regressions ❌  \(secondary\)    | 1\.0%   | \[0\.1%, 4\.6%\]      | 86    |
| Improvements ✅  \(primary\)     | \-3\.8% | \[\-7\.3%, \-0\.3%\]  | 2     |
| Improvements ✅  \(secondary\)   | \-2\.3% | \[\-18\.5%, \-0\.2%\] | 44    |
| All ❌✅ \(primary\)              | 1\.0%   | \[\-7\.3%, 9\.1%\]    | 125   |
• 2 Regressions, 4 Improvements, 10 Mixed; 7 of them in rollups 40 artifact comparisons made in total[Full report here](https://github.com/rust-lang/rustc-perf/blob/a63db4d1799853b334e4106d914fba24e49c8782/triage/2025/2025-06-24.md)[Approved RFCs](https://github.com/rust-lang/rfcs/commits/master)Changes to Rust follow the Rust [RFC \(request for comments\) process](https://github.com/rust-lang/rfcs#rust-rfcs)\. These are the RFCs that were approved for implementation this week:
• No RFCs were approved this week\.
• Final Comment PeriodEvery week, [the team](https://www.rust-lang.org/team.html) announces the 'final comment period' for RFCs and key PRs which are reaching a decision\. Express your opinions now\.Tracking Issues & PRs[Rust](https://github.com/rust-lang/rust/issues?q=is%3Aopen+label%3Afinal-comment-period+sort%3Aupdated-desc)
• [Use lld by default on x86\_64\-unknown\-linux\-gnu stable](https://github.com/rust-lang/rust/pull/140525)
• [Allow \#\[must\_use\] on associated types to warn on unused values in generic contexts](https://github.com/rust-lang/rust/pull/142590)
• [Fix proc\_macro::Ident 's handling of $crate](https://github.com/rust-lang/rust/pull/141996)
• [Ensure non\-empty buffers for large vectored I/O](https://github.com/rust-lang/rust/pull/138879)
• [Rust RFCs](https://github.com/rust-lang/rfcs/labels/final-comment-period)
• [RFC: \-\-crate\-attr](https://github.com/rust-lang/rfcs/pull/3791)
• No Items entered Final Comment Period this week for [Cargo](https://github.com/rust-lang/cargo/issues?q=is%3Aopen+label%3Afinal-comment-period+sort%3Aupdated-desc), [Language Reference](https://github.com/rust-lang/reference/issues?q=is%3Aopen+label%3Afinal-comment-period+sort%3Aupdated-desc), [Language Team](https://github.com/rust-lang/lang-team/issues?q=is%3Aopen+label%3Afinal-comment-period+sort%3Aupdated-desc+) or [Unsafe Code Guidelines](https://github.com/rust-lang/unsafe-code-guidelines/issues?q=is%3Aopen+label%3Afinal-comment-period+sort%3Aupdated-desc)\.Let us know if you would like your PRs, Tracking Issues or RFCs to be tracked as a part of this list\.[New and Updated RFCs](https://github.com/rust-lang/rfcs/pulls)
• No New or Updated RFCs were created this week\.
