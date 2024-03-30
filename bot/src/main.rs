use std::{env, sync::Arc};

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::GatewayIntents;
use serenity::model::gateway::Ready;
use serenity::prelude::{Client, Context, EventHandler, Mutex, TypeMapKey};
use tracing::{error, info};

use testauskameli::snippets::register_all;
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
        register_all(&executor);

        let (sender, receiver) = flume::unbounded();

        tokio::spawn(async move { executor.run(receiver).await });
        Self { sender }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mentioned = msg.mentions_me(&ctx.http).await.unwrap_or(false);

        let mut send_string = msg.content.to_string();
        if msg.author.bot || !mentioned && msg.attachments.is_empty() {
            return;
        }

        let mut needs_recoding = Vec::new();
        if !msg.attachments.is_empty() {
            for a in &msg.attachments {
                if let Some(ct) = &a.content_type {
                    if ct.starts_with("video/") && a.height.is_none() {
                        needs_recoding.push(a.url.clone());
                    }
                }
            }
        }

        if !needs_recoding.is_empty() {
            send_string = format!("```h264ify{}```", needs_recoding.join("\n"));
        } else if !mentioned {
            return;
        }

        if let Err(e) = self.sender.send_async((send_string, (ctx, msg))).await {
            error!("failed to send data to executor, is it running? [{}]", e);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(
        &token,
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT,
    )
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
