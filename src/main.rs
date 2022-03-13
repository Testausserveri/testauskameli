use std::io::Write;
use std::process::{Command, Stdio};

use std::{env, sync::Arc};

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let me = ctx.http.get_current_user().await.unwrap().id.0;
        if !msg.mentions.iter().any(|u| u.id.0 == me) {
            return;
        }
        let filetype = if msg.content.contains("```hs") {
            "```hs"
        } else if msg.content.contains("```haskell") {
            "```haskell"
        } else {
            return;
        };
        let code = msg.content[msg.content.find(filetype).unwrap() + filetype.len()
            ..msg.content.rfind("```").unwrap()]
            .to_string();
        info!("Running program: {}", &code);
        let mut runghc = Command::new("timeout")
            .args([
                "-k",
                &(env::var("KAMELI_TIMELIMIT")
                    .unwrap_or(String::from("10"))
                    .parse::<i32>()
                    .unwrap()
                    + 2)
                .to_string(),
                &env::var("KAMELI_TIMELIMIT").unwrap_or(String::from("10")),
                "s6-softlimit",
                "-a",
                &env::var("KAMELI_MEMLIMIT").unwrap_or(String::from("1000000000")),
                "-f",
                &env::var("KAMELI_FILELIMIT").unwrap_or(String::from("40000")),
                "-p",
                &env::var("KAMELI_PROCESSLIMIT").unwrap_or(String::from("10")),
                "sudo",
                "-u",
                &env::var("KAMELI_RUNUSER").unwrap_or(String::from("runhaskell")),
                "runghc",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir("/tmp")
            .spawn()
            .unwrap();
        let mut stdin = runghc.stdin.take().unwrap();
        std::thread::spawn(move || {
            stdin.write_all(code.as_bytes()).unwrap();
            drop(stdin);
        });
        let output = runghc.wait_with_output().unwrap();
        let mut stdout = String::from_utf8(output.stdout).unwrap();
        let mut stderr = String::from_utf8(output.stderr).unwrap();
        stdout.truncate(984);
        stderr.truncate(984);
        msg.reply(
            &ctx.http,
            format!("```\nstdout:\n{}```\n```\nstderr:\n{}```", stdout, stderr),
        )
        .await
        .unwrap();
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
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
