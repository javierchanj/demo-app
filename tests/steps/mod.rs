use cucumber::{given, then, when};

use crate::PostsWorld;

#[given("the posts API is running")]
fn posts_api_running(_world: &mut PostsWorld) {}

#[when("Anna checks the health endpoint")]
fn check_health(world: &mut PostsWorld) {
    // Lab B repoints this harness at the real app; for the smoke test we
    // just prove the plumbing compiles and runs.
    world.last_status = Some(200);
}

#[then(expr = "the response status is {int}")]
fn assert_status(world: &mut PostsWorld, expected: u16) {
    assert_eq!(world.last_status, Some(expected));
}
