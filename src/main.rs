#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]

mod data;
mod error;

use lumi::serenity_prelude::{self as serenity};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().unwrap();

    let options = lumi::FrameworkOptions {
        commands: moth_commands::commands(),
        prefix_options: lumi::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            additional_prefixes: vec![lumi::Prefix::Literal("m!"), lumi::Prefix::Literal("m")],
            edit_tracker: Some(Arc::new(lumi::EditTracker::for_timespan(
                Duration::from_secs(600),
            ))),
            ..Default::default()
        },

        on_error: |error| Box::pin(error::handler(error)),

        command_check: Some(|ctx| Box::pin(moth_commands::command_check(ctx))),

        skip_checks_for_owners: false,
        ..Default::default()
    };

    let framework = lumi::Framework::new(options);

    let token = serenity::Token::from_env("MOTH_TOKEN")
        .expect("Missing `MOTH_TOKEN` environment variable.");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let mut settings = serenity::Settings::default();
    settings.max_messages = 1000;

    let data = data::setup().await;

    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .data(data)
        .cache_settings(settings)
        .event_handler(moth_events::Handler)
        .await
        .unwrap();

    client.start().await.unwrap();
}
