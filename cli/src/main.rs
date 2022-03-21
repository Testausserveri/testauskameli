use std::env;

use tokio::fs::read_to_string;
use tokio::io::AsyncReadExt;
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

    let (sender, receiver) = flume::unbounded();
    let task = tokio::spawn(async move { executor.run(receiver).await });

    let arg = env::args().skip(1).next();

    let input = match (&arg).into_iter().map(|x| x.as_ref()).next() {
        None | Some("-") => {
            let mut stdin = tokio::io::stdin();
            let mut input = String::new();

            stdin
                .read_to_string(&mut input)
                .await
                .expect("failed to read from stdin");
            input
        }
        Some(file) => read_to_string(file).await.expect("failed to read file"),
    };

    sender
        .send_async((input, ()))
        .await
        .expect("BUG: impossible bruh");

    drop(sender);

    task.await.unwrap();
}
