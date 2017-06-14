#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate slack_api;

mod error {
    use slack_api::{chat, requests};

    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            SlackApi(chat::PostMessageError<requests::Error>);
        }
    }
}

use error::*;

use clap::{Arg, ArgGroup, App, AppSettings};
use slack_api::requests::Client;
use slack_api::chat;
use std::io;
use std::io::{Read};
use std::env;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }
    }
}

fn run() -> Result<()> {
    // require one of channel or user
    // optional: one of message or file or snippet
    //   but will default to stdin
    // sender name is optional
    let app_m = App::new("slackit-cli")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("get_token")
            .short("t")
            .long("token")
            .takes_value(true)
            .help("set token here or env var SLACK_API_TOKEN"))
        .arg(Arg::with_name("get_channel")
            .short("c")
            .long("channel")
            .takes_value(true)
            .help("channel to send to. Exclude #"))
        .arg(Arg::with_name("get_user")
            .short("u")
            .long("user")
            .takes_value(true)
            .help("user to send to. Exclude @"))
        .group(ArgGroup::with_name("target")
            .args(&["get_channel", "get_user"])
            .required(true))
        .arg(Arg::with_name("get_message")
            .short("m")
            .long("message")
            .takes_value(true)
            .help("text of message to send. stdin will be appended"))
//        .arg(Arg::with_name("get_snippet")
//             .short("s")
//             .long("snippet")
//             .help("text of snippet to send. stdin will be appended"))
//        .arg(Arg::with_name("get_file_upload")
//             .short("f")
//             .long("file")
//             .help("file to upload"))
        .arg(Arg::with_name("get_sender_name")
            .short("n")
            .long("name")
            .takes_value(true)
            .help("name to send as. Default bot"))
        .get_matches();

    // config
    let token = app_m.value_of("get_token")
        .map(|s| s.to_owned())
        .unwrap_or(env::var("SLACK_API_TOKEN")
            .chain_err(|| "No token found")?
        );

    // either channel or user has to be present, as encoded
    // in the cli config
    let mut target = String::new();
    if app_m.is_present("get_channel") {
        target.push_str("#");
        target.push_str(app_m.value_of("get_channel").unwrap());
    } else {
        target.push_str("@");
        target.push_str(app_m.value_of("get_user").unwrap());
    };

    // Optional configs
    let sender_name = app_m.value_of("get_sender_name");

    // Set up client and send message
    let client = Client::new().unwrap();

    // Message will always be sent, whether
    // through -m or through stdin or both
    let mut message = app_m.value_of("get_message")
        .map(|s| s.to_owned())
        .unwrap_or("".to_owned());

    let stdin = io::stdin();
    stdin.lock().read_to_string(&mut message)?;

    let message = format_slack_message(&message);
    let mut m = chat::PostMessageRequest::default();
    m.channel = &target;
    m.text = &message;
    m.username = sender_name;

    let _ = chat::post_message(
        &client,
        &token,
        &m,
    )?;

    // Now do file upload

    Ok(())
}

fn format_slack_message(s: &str) -> String {
    // need to format links and newlines.
    format!("{}", s)
}
