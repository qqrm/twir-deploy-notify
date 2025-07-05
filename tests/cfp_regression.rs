mod common;
#[allow(dead_code)]
#[path = "../src/generator.rs"]
mod generator;
#[allow(dead_code)]
#[path = "../src/parser.rs"]
mod parser;
#[allow(dead_code)]
#[path = "../src/validator.rs"]
mod validator;

use common::assert_valid_markdown;
use generator::generate_posts;

const CFP_SNIPPET: &str = r#"## Call for Participation; projects and speakers

### CFP - Projects

Always wanted to contribute to open-source projects but did not know where to start?
Every week we highlight some tasks from the Rust community for you to pick and get started!

Some of these tasks may also have mentors available, visit the task page for more information.

<!-- CFPs go here, use this format: * [project name - title of issue](URL to issue) -->
<!-- * [ - ]() -->
* [Continuwuity - Default room ACLs](https://forgejo.ellis.link/continuwuation/continuwuity/issues/775)
* [Continuwuity - Ability to entirely disable typing and read receipts](https://forgejo.ellis.link/continuwuation/continuwuity/issues/821)
* [Continuwuity - bug: appservice users are not created on registration](https://forgejo.ellis.link/continuwuation/continuwuity/issues/813)
* [Continuwuity - Invite filtering / disable invites per account](https://forgejo.ellis.link/continuwuation/continuwuity/issues/836)
<!-- or if none - *No Calls for participation were submitted this week.* -->

If you are a Rust project owner and are looking for contributors, please submit tasks [here][guidelines] or through a [PR to TWiR](https://github.com/rust-lang/this-week-in-rust) or by reaching out on [X (formerly Twitter)](https://x.com/ThisWeekInRust) or [Mastodon](https://mastodon.social/@thisweekinrust)!

[guidelines]:https://github.com/rust-lang/this-week-in-rust?tab=readme-ov-file#call-for-participation-guidelines

### CFP - Events

Are you a new or experienced speaker looking for a place to share something cool? This section highlights events that are being planned and are accepting submissions to join their event as a speaker.

<!-- CFPs go here, use this format: * [**event name**](URL to CFP)| Date CFP closes in YYYY-MM-DD | city,state,country | Date of event in YYYY-MM-DD -->
<!-- or if none - *No Calls for papers or presentations were submitted this week.* -->
*No Calls for papers or presentations were submitted this week.*

If you are an event organizer hoping to expand the reach of your event, please submit a link to the website through a [PR to TWiR](https://github.com/rust-lang/this-week-in-rust) or by reaching out on [X (formerly Twitter)](https://x.com/ThisWeekInRust) or [Mastodon](https://mastodon.social/@thisweekinrust)!
"#;

#[test]
fn cfp_section_generates_valid_markdown() {
    let input = format!("Title: Test\nNumber: 1\nDate: 2025-06-25\n\n{CFP_SNIPPET}");
    let posts = generate_posts(input).unwrap();
    assert!(!posts.is_empty());
    for post in &posts {
        assert_valid_markdown(post);
    }
}
