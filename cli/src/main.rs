use tracing::*;

use testauskameli::snippets::register_all;
use testauskameli::Executor;

mod executor;
use crate::executor::CliExecutor;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();
    info!("starting kameli cli");

    let executor = CliExecutor::new();
    register_all(&executor);
    info!("executor started");

    let (_sender, receiver) = flume::unbounded();
    tokio::spawn(async move { executor.run(receiver).await });

    loop {
        todo!("Get and send input");
    }
}
