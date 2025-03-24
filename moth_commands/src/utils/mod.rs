pub mod checks;
pub mod pagination;

use std::time::Duration;

pub use checks::*;
use lumi::CreateReply;
use moth_core::data::structs::{Context, Error};
pub use pagination::*;

pub async fn handle_cooldown(remaining_cooldown: Duration, ctx: Context<'_>) -> Result<(), Error> {
    let msg = format!(
        "You're too fast. Please wait {} seconds before retrying",
        remaining_cooldown.as_secs()
    );
    ctx.send(CreateReply::default().content(msg).ephemeral(true))
        .await?;

    Ok(())
}

pub async fn bot_permissions(ctx: crate::Context<'_>) -> Result<serenity::all::Permissions, Error> {
    match ctx {
        lumi::Context::Application(actx) => Ok(actx.interaction.app_permissions),
        lumi::Context::Prefix(pctx) => prefix_member_perms(pctx).await,
    }
}

pub async fn prefix_bot_perms(
    ctx: crate::PrefixContext<'_>,
) -> Result<serenity::all::Permissions, Error> {
    let Some(guild) = ctx.guild() else {
        return Err("Could not retrieve guild from cache.".into());
    };

    let channel_id = ctx.channel_id();
    let (channel, is_thread) = match guild.channels.get(&channel_id) {
        Some(channel) => (channel, false),
        None => guild
            .threads
            .iter()
            .find(|c| c.id == channel_id)
            .map(|channel| (channel, true))
            .expect("Channels or threads are always sent alongside the guild."),
    };

    let mut permissions = guild.user_permissions_in(
        channel,
        guild
            .members
            .get(&ctx.serenity_context().cache.current_user().id)
            .expect("Bot member is always present in the guild cache."),
    );

    if is_thread && permissions.send_messages_in_threads() {
        permissions |= serenity::all::Permissions::SEND_MESSAGES;
    }

    Ok(permissions)
}

pub async fn prefix_member_perms(
    ctx: crate::PrefixContext<'_>,
) -> Result<serenity::all::Permissions, Error> {
    let Some(guild) = ctx.guild() else {
        return Err("Could not retrieve guild from cache.".into());
    };

    let channel_id = ctx.channel_id();
    let (channel, is_thread) = match guild.channels.get(&channel_id) {
        Some(channel) => (channel, false),
        None => guild
            .threads
            .iter()
            .find(|c| c.id == channel_id)
            .map(|channel| (channel, true))
            .expect("Channels or threads are always sent alongside the guild."),
    };

    let mut permissions = guild.partial_member_permissions_in(
        channel,
        ctx.author().id,
        ctx.msg
            .member
            .as_ref()
            .expect("PartialMember is always present on a message from a guild."),
    );

    if is_thread && permissions.send_messages_in_threads() {
        permissions |= serenity::all::Permissions::SEND_MESSAGES;
    }

    Ok(permissions)
}
