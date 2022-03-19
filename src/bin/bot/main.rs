use std::{env, sync::Arc};

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};

use testauskameli::Executor;

mod executor;
use crate::executor::DiscordExecutor;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler {
    sender: flume::Sender<(String, <DiscordExecutor as Executor>::Context)>,
}

impl Handler {
    fn new() -> Self {
        let executor = DiscordExecutor::new();
        let (sender, receiver) = flume::unbounded();

        tokio::spawn(async move { executor.run(receiver).await });

        Self { sender }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || !msg.mentions_me(&ctx.http).await.unwrap_or(false) {
            return;
        }

        if let Err(e) = self
            .sender
            .send_async((msg.content.to_string(), (ctx, msg)))
            .await
        {
            error!("failed to send data to executor, is it running? [{}]", e);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler::new())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
