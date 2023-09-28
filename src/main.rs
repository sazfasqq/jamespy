mod commands;
use commands::*;
mod database;
mod event_handler;
mod event_handlers;
mod utils;

use database::init_data;
use database::init_redis_pool;
use poise::serenity_prelude as serenity;
use std::{env::var, time::Duration};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub db: database::DbPool,
    pub redis: database::RedisPool,
    time_started: std::time::Instant,
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let db_pool = init_data().await;
    let redis_pool = init_redis_pool().await;

    let options = poise::FrameworkOptions {
        commands: vec![
            register(),
            owner::other::shutdown(),
            owner::other::say(),
            owner::other::dm(),
            owner::other::react(),
            owner::cache::cached_users_raw(),
            owner::cache::cached_users(),
            owner::cache::max_messages(),
            owner::cache::cache_stats(),
            owner::presence::status(),
            owner::presence::reset_presence(),
            owner::presence::set_activity(),
            owner::database::dbstats(),
            owner::database::sql(),
            owner::cache::guild_message_cache(),
            owner::lists::update_lists(),
            owner::lists::unload_lists(),
            meta::source(),
            meta::about(),
            meta::help(),
            meta::uptime(),
            meta::ping(),
            meta::toggle(),
            general::lob::lob(),
            general::lob::reload_lob(),
            general::lob::no_lob(),
            general::lob::new_lob(),
            general::lob::delete_lob(),
            general::lob::total_lobs(),
            general::lob::send_lobs(),
            utility::snippets::set_snippet(),
            utility::snippets::snippet(),
            utility::snippets::list_snippets(),
            utility::snippets::remove_snippet(),
            utility::random::choose(),
            utility::users::guild_flags(),
            utility::users::last_reactions(),
            utility::users::statuses(),
            utility::info::role_info(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(600))),
            ..Default::default()
        },

        on_error: |error| Box::pin(on_error(error)),

        skip_checks_for_owners: false,
        event_handler: |event: &serenity::FullEvent, framework, data| {
            Box::pin(event_handler::event_handler(event.clone(), framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::new(options, move |ctx, ready, framework| {
        Box::pin(async move {
            println!("Logged in as {}", ready.user.name);
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data {
                db: db_pool.clone(),
                redis: redis_pool.clone(),
                time_started: std::time::Instant::now(),
            })
        })
    });

    let token = var("JAMESPY_TOKEN").expect("Missing `JAMESPY_TOKEN` env var. Aborting...");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();
}
