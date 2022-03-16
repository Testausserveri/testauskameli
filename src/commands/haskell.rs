use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

fn cleanup(msgid: u64) {
    std::fs::remove_file(format!("/tmp/{}.hs", msgid)).ok();
    std::fs::remove_file(format!("/tmp/{}", msgid)).ok();
    std::fs::remove_file(format!("/tmp/{}.hi", msgid)).ok();
    std::fs::remove_file(format!("/tmp/{}.o", msgid)).ok();
}

pub async fn compile_and_run(ctx: &Context, msg: Message, code: &str) {
    let mut file = File::create(format!("/tmp/{}.hs", msg.id.0)).unwrap();
    file.write_all(code.as_bytes()).unwrap();
    let ghc = Command::new("ghc")
        .arg("-o")
        .arg(msg.id.0.to_string())
        .args(&mut env::var("GHC_ARGS").unwrap_or_default().split_whitespace())
        .arg(format!("{}.hs", msg.id.0))
        .stderr(Stdio::piped())
        .current_dir("/tmp")
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    if !ghc.status.success() {
        msg.reply(
            &ctx.http,
            format!(
                "Error compiling the code:\n```\n{}```",
                String::from_utf8(ghc.stderr).unwrap()
            ),
        )
        .await
        .unwrap();
        cleanup(msg.id.0);
        return;
    }
    let runghc = Command::new("sudo")
        .args([
            "-u",
            &env::var("KAMELI_RUNUSER").unwrap_or(String::from("runhaskell")),
            "timeout",
            "-s",
            "KILL",
            &env::var("KAMELI_TIMELIMIT").unwrap_or(String::from("10")),
            "s6-softlimit",
            "-a",
            &env::var("KAMELI_MEMLIMIT").unwrap_or(String::from("1000000000")),
            "-f",
            &env::var("KAMELI_FILELIMIT").unwrap_or(String::from("40000")),
            "-p",
            &env::var("KAMELI_PROCESSLIMIT").unwrap_or(String::from("1")),
            &format!("./{}", msg.id.0),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir("/tmp")
        .spawn()
        .unwrap();
    let output = runghc.wait_with_output().unwrap();
    let mut stdout = String::from_utf8(output.stdout).unwrap();
    let mut stderr = String::from_utf8(output.stderr).unwrap();
    stderr.truncate(1950);
    if !output.status.success() {
        msg.reply(
            &ctx.http,
            format!("Code ran unsuccessfully\n```\n{}```", stderr),
        )
        .await
        .unwrap();
    } else {
        stdout.truncate(1984);
        msg.reply(&ctx.http, format!("output\n```\n{}```", stdout))
            .await
            .unwrap();
    }
    cleanup(msg.id.0);
}
