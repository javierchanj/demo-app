mod steps;

use cucumber::World;

#[derive(Debug, Default, cucumber::World)]
pub struct PostsWorld {
    pub last_status: Option<u16>,
}

#[tokio::main]
async fn main() {
    PostsWorld::run("tests/features/smoke.feature").await;
}
