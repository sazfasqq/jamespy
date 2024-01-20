use crate::helper::{
    auto_archive_duration_to_string, channel_type_to_string, forum_layout_to_string,
    get_guild_name, get_permission_changes, sort_order_to_string,
};

use crate::{Data, Error};

use poise::serenity_prelude::audit_log::Action::VoiceChannelStatus;
use poise::serenity_prelude::{
    self as serenity, ChannelFlags, ChannelId, ChannelType, CreateEmbed, ForumEmoji, GuildChannel,
    GuildId, PartialGuildChannel, UserId, VoiceChannelStatusAction,
};

use std::time::Duration;

pub async fn channel_create(ctx: &serenity::Context, channel: &GuildChannel) -> Result<(), Error> {
    let guild_name = get_guild_name(ctx, Some(channel.guild_id));

    let kind = channel_type_to_string(channel.kind);
    println!(
        "\x1B[34m[{}] #{} ({}) was created!\x1B[0m",
        guild_name, channel.name, kind
    );
    Ok(())
}

pub async fn channel_update(
    ctx: &serenity::Context,
    old: &Option<GuildChannel>,
    new: &GuildChannel,
) -> Result<(), Error> {
    let mut channel_name = String::new();
    let mut kind = String::new();
    let mut diff = String::new();

    let guild_name = get_guild_name(ctx, Some(new.guild_id));

    if let Some(old) = old {
        channel_name = new.name.to_string();
        kind = channel_type_to_string(new.kind);

        // Differences
        if old.name != new.name {
            diff.push_str(&format!("Name: {} -> {}\n", old.name, new.name));
        }
        if old.nsfw != new.nsfw {
            diff.push_str(&format!("NSFW: {} -> {}\n", old.nsfw, new.nsfw));
        }

        // Check if the channel is in a category.
        match (old.parent_id, new.parent_id) {
            (Some(old_parent_id), Some(new_parent_id)) if old_parent_id != new_parent_id => {
                diff.push_str(&format!(
                    "Parent: {} -> {}\n",
                    old_parent_id.name(ctx).await?,
                    new_parent_id.name(ctx).await?
                ));
            }
            (None, Some(parent_id)) => {
                diff.push_str(&format!("Parent: None -> {}\n", parent_id.name(ctx).await?));
            }
            (Some(parent_id), None) => {
                diff.push_str(&format!("Parent: {} -> None\n", parent_id.name(ctx).await?));
            }
            _ => {}
        }

        match (old.bitrate, new.bitrate) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!(
                    "Bitrate: {}kbps -> {}kbps\n",
                    u32::from(old_value) / 1000,
                    u32::from(new_value) / 1000
                ));
            }
            _ => {}
        }

        // TODO: show if permission overrides have been removed.
        if old.permission_overwrites != new.permission_overwrites {
            for old_overwrite in &old.permission_overwrites {
                for new_overwrite in &new.permission_overwrites {
                    if old_overwrite.kind == new_overwrite.kind {
                        let changes_str = get_permission_changes(
                            ctx.clone(),
                            new.guild_id,
                            old_overwrite.allow,
                            new_overwrite.allow,
                            old_overwrite.deny,
                            new_overwrite.deny,
                            new_overwrite.kind,
                        )
                        .await;
                        diff.push_str(&changes_str);
                    }
                }
            }
        }

        // If both the old and new topic are the same, it shouldn't print.
        match (&old.topic, &new.topic) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!("Topic: {old_value} -> {new_value}\n"));
            }
            (None, Some(new_value)) if !new_value.is_empty() => {
                diff.push_str(&format!("Topic: None -> {new_value}\n"));
            }
            (Some(old_value), None) if !old_value.is_empty() => {
                diff.push_str(&format!("Topic: {old_value} -> None\n"));
            }
            _ => {}
        }

        match (old.user_limit, new.user_limit) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!("User Limit: {old_value} -> {new_value}\n"));
            }
            (None, Some(new_value)) => {
                diff.push_str(&format!("User Limit: None -> {new_value}\n"));
            }
            (Some(old_value), None) => {
                diff.push_str(&format!("User Limit: {old_value} -> None\n"));
            }
            _ => {}
        }

        match (old.rate_limit_per_user, new.rate_limit_per_user) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!("Slowmode: {old_value}s -> {new_value}s\n"));
            }
            _ => {}
        }

        match (
            old.default_thread_rate_limit_per_user,
            new.default_thread_rate_limit_per_user,
        ) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!(
                    "Default Thread Slowmode: {old_value}s -> {new_value}s\n"
                ));
            }
            _ => {}
        }

        match (
            old.default_auto_archive_duration,
            new.default_auto_archive_duration,
        ) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                let old_duration = auto_archive_duration_to_string(old_value);
                let new_duration = auto_archive_duration_to_string(new_value);
                diff.push_str(&format!(
                    "Default Archive Duration: {old_duration} -> {new_duration}\n"
                ));
            }
            _ => {}
        }

        match (&old.default_reaction_emoji, &new.default_reaction_emoji) {
            (Some(ForumEmoji::Name(old_name)), Some(ForumEmoji::Name(new_name)))
                if old_name != new_name =>
            {
                diff.push_str(&format!(
                    "Default Reaction Emoji: {old_name} -> {new_name}\n"
                ));
            }
            (None, Some(ForumEmoji::Name(new_name))) => {
                diff.push_str(&format!("Default Reaction Emoji: None -> {new_name}\n"));
            }
            (Some(ForumEmoji::Name(old_name)), None) => {
                diff.push_str(&format!("Default Reaction Emoji: {old_name} -> None\n"));
            }
            _ => {}
        }

        if old.flags.contains(ChannelFlags::REQUIRE_TAG)
            != new.flags.contains(ChannelFlags::REQUIRE_TAG)
        {
            if new.flags.contains(ChannelFlags::REQUIRE_TAG) {
                diff.push_str("REQUIRE_TAG was enabled!");
            } else {
                diff.push_str("REQUIRE_TAG was disabled!");
            }
        }

        match (old.default_forum_layout, new.default_forum_layout) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!(
                    "Default Forum Layout: {} -> {}\n",
                    forum_layout_to_string(old_value),
                    forum_layout_to_string(new_value)
                ));
            }
            (None, Some(new_value)) => {
                diff.push_str(&format!(
                    "Default Forum Layout: None -> {}\n",
                    forum_layout_to_string(new_value)
                ));
            }
            (Some(old_value), None) => {
                diff.push_str(&format!(
                    "Default Forum Layout: {} -> None\n",
                    forum_layout_to_string(old_value)
                ));
            }
            _ => {}
        }

        match (old.default_sort_order, new.default_sort_order) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!(
                    "Default Forum Layout: {} -> {}\n",
                    sort_order_to_string(old_value),
                    sort_order_to_string(new_value)
                ));
            }
            (None, Some(new_value)) => {
                diff.push_str(&format!(
                    "Default Forum Layout: None -> {}\n",
                    sort_order_to_string(new_value)
                ));
            }
            (Some(old_value), None) => {
                diff.push_str(&format!(
                    "Default Forum Layout: {} -> None\n",
                    sort_order_to_string(old_value)
                ));
            }

            _ => {}
        }
        // Forum tags doesn't implement what i want, I refuse to do it until this is matched.
    }
    diff = diff.trim_end_matches('\n').to_string();
    if !diff.is_empty() {
        println!("\x1B[34m[{guild_name}] #{channel_name} was updated! ({kind})\x1B[0m\n{diff}");
    }
    Ok(())
}

pub async fn channel_delete(ctx: &serenity::Context, channel: &GuildChannel) -> Result<(), Error> {
    let kind = channel_type_to_string(channel.kind);
    let guild_name = get_guild_name(ctx, Some(channel.guild_id));

    println!(
        "\x1B[34m[{}] #{} ({}) was deleted!\x1B[0m",
        guild_name, channel.name, kind
    );

    Ok(())
}

pub async fn thread_create(ctx: &serenity::Context, thread: &GuildChannel) -> Result<(), Error> {
    let guild_id = thread.guild_id;
    let guild_name = get_guild_name(ctx, Some(guild_id));
    let kind = channel_type_to_string(thread.kind);

    let parent_channel_name = if let Some(parent_id) = thread.parent_id {
        parent_id.name(ctx).await?
    } else {
        "Unknown Channel".to_string()
    };

    println!(
        "\x1B[94m[{}] Thread #{} ({}) was created in #{}!\x1B[0m",
        guild_name, thread.name, kind, parent_channel_name
    );
    Ok(())
}

pub async fn thread_update(
    ctx: &serenity::Context,
    old: &Option<GuildChannel>,
    new: &GuildChannel,
) -> Result<(), Error> {
    let guild_id = new.guild_id;
    let guild_name = get_guild_name(ctx, Some(guild_id));
    let kind = channel_type_to_string(new.kind);
    let mut diff = String::new();

    let parent_channel_name = if let Some(parent_id) = new.parent_id {
        parent_id.name(ctx).await?
    } else {
        "Unknown Channel".to_string()
    };

    if let Some(old) = old {
        if old.name != new.name {
            diff.push_str(&format!("Name: {} -> {}\n", old.name, new.name));
        }

        match (old.rate_limit_per_user, new.rate_limit_per_user) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!("Slowmode: {old_value}s -> {new_value}s\n"));
            }
            _ => {}
        }

        if old.flags.contains(ChannelFlags::PINNED) != new.flags.contains(ChannelFlags::PINNED) {
            if new.flags.contains(ChannelFlags::PINNED) {
                diff.push_str("Pinned: true");
            } else {
                diff.push_str("Pinned: false");
            }
        }

        if let (Some(old_metadata), Some(new_metadata)) = (old.thread_metadata, new.thread_metadata)
        {
            if old.kind == ChannelType::PrivateThread {
                match (old_metadata.invitable(), new_metadata.invitable()) {
                    (true, false) => diff.push_str("Invitable: false\n"),
                    (false, true) => diff.push_str("Invitable: true\n"),
                    _ => {}
                }
            }

            match (old_metadata.archived(), new_metadata.archived()) {
                (true, false) => diff.push_str("Archived: false\n"),
                (false, true) => diff.push_str("Archived: true\n"),
                _ => {}
            }
            match (old_metadata.locked(), new_metadata.locked()) {
                (true, false) => diff.push_str("Locked: false\n"),
                (false, true) => diff.push_str("Locked: true\n"),
                _ => {}
            }

            if old_metadata.auto_archive_duration != new_metadata.auto_archive_duration {
                let old_duration =
                    auto_archive_duration_to_string(old_metadata.auto_archive_duration);
                let new_duration =
                    auto_archive_duration_to_string(new_metadata.auto_archive_duration);
                diff.push_str(&format!(
                    "Archive Duration: {old_duration} -> {new_duration}\n"
                ));
            }
        }
    }

    diff = diff.trim_end_matches('\n').to_string();
    if !diff.is_empty() {
        println!(
            "\x1B[94m[{}] #{} in {} was updated! ({})\x1B[0m\n{}",
            guild_name, new.name, parent_channel_name, kind, diff
        );
    }

    Ok(())
}

pub async fn thread_delete(
    ctx: &serenity::Context,
    thread: &PartialGuildChannel,
    full_thread_data: &Option<GuildChannel>,
) -> Result<(), Error> {
    let guild_id = thread.guild_id;
    let mut channel_name = String::new();
    let mut parent_channel_name: String = String::new();
    let mut kind = String::new();
    let guild_name = get_guild_name(ctx, Some(guild_id));

    if let Some(full_thread) = full_thread_data {
        channel_name = full_thread.name.to_string();
        kind = channel_type_to_string(full_thread.kind);

        if let Some(parent_id) = full_thread.parent_id {
            parent_channel_name = parent_id.name(ctx).await?;
        } else {
            parent_channel_name = "Unknown Channel".to_string();
        }
    }

    if channel_name.is_empty() {
        println!("\x1B[94m[{guild_name}] An unknown thread was deleted!\x1B[0m");
    } else {
        println!(
            "\x1B[94m[{guild_name}] Thread #{channel_name} ({kind}) was deleted from \
             #{parent_channel_name}!\x1B[0m"
        );
    }
    Ok(())
}

pub async fn voice_channel_status_update(
    ctx: &serenity::Context,
    old: &Option<String>,
    status: &Option<String>,
    id: &ChannelId,
    guild_id: &GuildId,
    data: &Data,
) -> Result<(), Error> {
    let vcstatus = {
        let config = data.config.read().unwrap();
        config.vcstatus.clone()
    };
    if vcstatus.action {
        let old_field: Option<String>;
        let new_field: Option<String>;
        match (old, status.clone()) {
            (None, None) => {
                old_field = None;
                new_field = None;
                add(ctx, id, guild_id, old_field, new_field, status, data).await?;
            }
            (Some(old), Some(status)) => {
                old_field = Some(old.to_string());
                new_field = Some(status.clone());
                if old_field != new_field {
                    add(ctx, id, guild_id, old_field, new_field, &Some(status), data).await?;
                }
            }
            (None, Some(status)) => {
                old_field = None;
                new_field = Some(status.clone());
                add(ctx, id, guild_id, old_field, new_field, &Some(status), data).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn add(
    ctx: &serenity::Context,
    id: &ChannelId,
    guild_id: &GuildId,
    old_field: Option<String>,
    new_field: Option<String>,
    status: &Option<String>,
    data: &Data,
) -> Result<(), Error> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    let logs = guild_id
        .audit_logs(
            &ctx,
            Some(VoiceChannelStatus(VoiceChannelStatusAction::StatusUpdate)),
            None,
            None,
            Some(5),
        )
        .await?;
    let mut user_id: Option<UserId> = None;

    for log in &logs.entries {
        if log.options.is_none() {
            continue;
        }

        let options = &log.options.as_ref().unwrap();

        if let Some(str) = &options.status {
            if str == status.as_deref().unwrap_or_default() && options.channel_id == Some(*id) {
                user_id = Some(log.user_id);
                break;
            }
        }
    }

    // TODO: possibly improve.
    if user_id.is_none() {
        return Ok(());
    }

    let user_id = user_id.unwrap();

    let vcstatus = {
        let config = data.config.read().unwrap();
        config.vcstatus.clone()
    };

    // check if regex for blacklists exist and if a new status exists.
    let blacklisted = if let (Some(regex_patterns), Some(value)) = (&vcstatus.regex, &new_field) {
        check_blacklisted(value, regex_patterns).await
    } else {
        false
    };

    post_messages(
        ctx,
        data,
        *id,
        old_field.as_deref(),
        new_field.as_deref(),
        user_id,
        blacklisted,
    )
    .await?;
    Ok(())
}

async fn check_blacklisted(msg: &str, patterns: &[regex::Regex]) -> bool {
    patterns.iter().any(|pattern| pattern.is_match(msg))
}

async fn post_messages(
    ctx: &serenity::Context,
    data: &Data,
    id: ChannelId,
    old: Option<&str>,
    new: Option<&str>,
    user_id: UserId,
    blacklisted: bool,
) -> Result<(), Error> {
    let channel_str: &str = &format!("<#{}>", id.get());

    let old_field = match old {
        Some(value) if !value.is_empty() => ("Old", value, true),
        _ => ("Old", "None", true),
    };

    let new_field = match new {
        Some(value) if !value.is_empty() => ("New", value, true),
        _ => ("New", "None", true),
    };

    let fields = [("Channel", channel_str, true), old_field, new_field];

    let user: serenity::User = user_id.to_user(&ctx).await.unwrap();
    let author_title = format!("{} changed a channel status", user.name);
    let author = serenity::CreateEmbedAuthor::new(author_title).icon_url(user.face());
    let footer = serenity::CreateEmbedFooter::new(format!(
        "User ID: {} • Please check user manually in audit log.",
        user.id.get()
    ));

    let embed = serenity::CreateEmbed::default()
        .fields(fields)
        .author(author)
        .footer(footer);

    send_msgs(ctx, data, user_id, embed, blacklisted).await?;

    Ok(())
}

async fn send_msgs(
    ctx: &serenity::Context,
    data: &Data,
    user_id: UserId,
    embed: CreateEmbed<'_>,
    blacklisted: bool,
) -> Result<(), Error> {
    let (post, announce) = {
        let status = &data.config.read().unwrap().vcstatus;
        (status.post_channel, status.announce_channel)
    };

    let content = if blacklisted {
        format!("<@{user_id}>: **Blacklisted word in status!**")
    } else {
        format!("<@{user_id}>")
    };

    if blacklisted {
        if let Some(announce) = announce {
            announce
                .send_message(
                    ctx,
                    serenity::CreateMessage::default()
                        .content(&content)
                        .embed(embed.clone()),
                )
                .await?;
        }
    }

    if let Some(post) = post {
        post.send_message(
            ctx,
            serenity::CreateMessage::default()
                .content(&content)
                .embed(embed),
        )
        .await?;
    }

    Ok(())
}