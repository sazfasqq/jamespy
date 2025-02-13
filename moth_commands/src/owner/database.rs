use crate::{owner::owner, Context, Error};
use moth_ansi::RESET;
use poise::serenity_prelude::{self as serenity, CreateEmbedFooter};
use sqlx::{query, Pool, Postgres, Row};
use std::fmt::Write;

#[poise::command(
    rename = "dbstats",
    aliases("db-stats", "db-info"),
    prefix_command,
    category = "Owner - Database",
    check = "owner",
    hide_in_help
)]
pub async fn dbstats(ctx: Context<'_>) -> Result<(), Error> {
    let db_pool = &ctx.data().database.db;

    let messages_tables = [
        ("messages", "message_id"),
        ("message_edits", "message_id"),
        ("message_deletion", "message_id"),
    ];

    let expressions = [
        ("stickers", "sticker_id"),
        ("sticker_usage", "id"),
        ("emotes", "id"),
        ("emote_usage", "id"),
    ];
    let misc_tables = [("dm_activity", "user_id"), ("starboard", "id")];

    let mut embed = serenity::CreateEmbed::default().title("Database Stats");

    let (Ok(messages_info), Ok(expressions_info), Ok(misc_info)) = tokio::join!(
        query_table_info(db_pool, &messages_tables),
        query_table_info(db_pool, &expressions),
        query_table_info(db_pool, &misc_tables),
    ) else {
        ctx.say("Failed to query information.").await?;
        return Ok(());
    };

    embed = embed.field("Messages", messages_info, true);
    embed = embed.field("Names", expressions_info, true);
    embed = embed.field("Miscellaneous", misc_info, true);

    let db_size_query = "SELECT pg_database_size(current_database())";
    let row = query(db_size_query).fetch_one(db_pool).await?;
    let db_size_bytes: i64 = row.get(0);
    let db_size = format!("{:.2} MB", db_size_bytes / (1024 * 1024));

    embed = embed.footer(CreateEmbedFooter::new(format!("Database size: {db_size}")));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn query_table_info(
    db_pool: &Pool<Postgres>,
    tables: &[(&str, &str)],
) -> Result<String, Error> {
    let mut info = String::new();

    for (table_name, pk_column) in tables {
        let sql_query = format!("SELECT COUNT({pk_column}) FROM {table_name}");
        let row = query(&sql_query).fetch_one(db_pool).await?;

        let count: i64 = row.get(0);

        writeln!(info, "**{table_name}**\n{count}").unwrap();
    }

    Ok(info)
}

#[poise::command(
    rename = "sql",
    prefix_command,
    category = "Owner - Database",
    owners_only,
    hide_in_help
)]
#[allow(clippy::similar_names)] // "now" and "row" are too close.
pub async fn sql(
    ctx: Context<'_>,
    #[description = "SQL query"]
    #[rest]
    query: String,
) -> Result<(), Error> {
    //TODO: completely overhaul this.
    let sql_query = query;
    let db_pool = &ctx.data().database.db;

    let now = std::time::Instant::now();

    println!("\x1B[31;40mWARNING: SQL COMMAND WAS TRIGGERED{RESET}");

    let result = sqlx::query(&sql_query).fetch_optional(db_pool).await;

    let elapsed = now.elapsed().as_millis();

    match result {
        Ok(Some(row)) => {
            if row.len() == 1 && row.try_get::<i64, _>(0).is_ok() {
                let count = row.get::<i64, _>(0);
                let formatted = format!("Counted {count} rows in {elapsed}ms");
                let message = poise::CreateReply::default().content(formatted);
                ctx.send(message).await?;
            } else {
                let formatted = format!("Query executed successfully in {elapsed}ms");
                let message = poise::CreateReply::default().content(formatted);
                ctx.send(message).await?;
            }
        }
        Ok(None) => {
            let formatted = format!("Query executed successfully in {elapsed}ms");
            let message = poise::CreateReply::default().content(formatted);
            ctx.send(message).await?;
        }
        Err(err) => {
            let error_message = format!("Error executing query: {err:?}");
            let message = poise::CreateReply::default().content(error_message);
            ctx.send(message).await?;
        }
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [dbstats(), sql()]
}
