use std::{borrow::Cow, sync::Arc};

use lumi::serenity_prelude as serenity;
use moth_core::data::responses::DetectionType;

pub async fn response_handler(ctx: &serenity::Context, msg: &serenity::Message) {
    if msg.author.id == ctx.cache.current_user().id {
        return;
    }

    let Some(guild_id) = msg.guild_id else { return };

    let data: std::sync::Arc<crate::Data> = ctx.data();

    let Ok(Some(regexes)) = data.database.get_responses_regexes(guild_id).await else {
        return;
    };

    let mut text_response = Vec::new();
    let mut emoji_responses = Vec::new();

    let mut requires_ocr = false;

    // Check if OCR is needed
    if let Some(channel_regexes) = regexes.channel.get(&msg.channel_id) {
        for regex in channel_regexes {
            if regex.detection_type.contains(DetectionType::OCR) {
                requires_ocr = true;
                break;
            }
        }
    }

    for regex in &regexes.global {
        if regex.detection_type.contains(DetectionType::OCR) {
            requires_ocr = true;
            break;
        }
    }

    // Process OCR once if needed
    let mut ocr_texts = Vec::new();
    if requires_ocr {
        ocr_texts = process_ocr_attachments(&data, msg).await;
    }

    // Process text-based detection
    if let Some(channel_regexes) = regexes.channel.get(&msg.channel_id) {
        for regex in channel_regexes {
            if regex.detection_type.contains(DetectionType::CONTENT) {
                let content = extract_message_content(msg);
                if regex.pattern.is_match(&content) {
                    match &regex.response {
                        moth_core::data::responses::ResponseType::Message(msg) => {
                            text_response.push(msg.clone());
                        }
                        moth_core::data::responses::ResponseType::Emoji(reaction_type) => {
                            emoji_responses.push(reaction_type.clone());
                        }
                    }
                }
            }

            if regex.detection_type.contains(DetectionType::OCR) {
                for ocr_text in &ocr_texts {
                    if regex.pattern.is_match(ocr_text) {
                        match &regex.response {
                            moth_core::data::responses::ResponseType::Message(msg) => {
                                text_response.push(msg.clone());
                            }
                            moth_core::data::responses::ResponseType::Emoji(reaction_type) => {
                                emoji_responses.push(reaction_type.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    for regex in &regexes.global {
        if regex.exceptions.contains(&msg.channel_id) {
            continue;
        }

        let (is_channel, category_id) = is_channel(ctx, msg);

        if !is_channel && !regex.recurse_threads {
            continue;
        }

        if let Some(category_id) = category_id {
            if regex.exceptions.contains(&category_id.widen()) {
                continue;
            }
        }

        if regex.detection_type.contains(DetectionType::CONTENT) {
            let content = extract_message_content(msg);
            if regex.pattern.is_match(&content) {
                match &regex.response {
                    moth_core::data::responses::ResponseType::Message(msg) => {
                        text_response.push(msg.clone());
                    }
                    moth_core::data::responses::ResponseType::Emoji(reaction_type) => {
                        emoji_responses.push(reaction_type.clone());
                    }
                }
            }
        }

        if regex.detection_type.contains(DetectionType::OCR) {
            for ocr_text in &ocr_texts {
                if regex.pattern.is_match(ocr_text) {
                    match &regex.response {
                        moth_core::data::responses::ResponseType::Message(msg) => {
                            text_response.push(msg.clone());
                        }
                        moth_core::data::responses::ResponseType::Emoji(reaction_type) => {
                            emoji_responses.push(reaction_type.clone());
                        }
                    }
                }
            }
        }
    }

    if let Ok(perms) = prefix_bot_perms(ctx, guild_id, msg.channel_id) {
        if perms.send_messages() {
            let response = text_response.join("\n\n");

            let _ = msg
                .reply(&ctx.http, response.chars().take(2000).collect::<String>())
                .await;
        }

        if perms.add_reactions() {
            for reaction in emoji_responses.iter().take(3) {
                match msg.react(&ctx.http, reaction.clone()).await {
                    Ok(()) => {}
                    Err(e) => println!("error while reacting: {e}"),
                }
            }
        }
    }
}

// TODO: dedupe other prefix_bot_perms
pub fn prefix_bot_perms(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    channel_id: serenity::GenericChannelId,
) -> Result<serenity::all::Permissions, crate::Error> {
    let Some(guild) = ctx.cache.guild(guild_id) else {
        return Err("Could not retrieve guild from cache.".into());
    };

    let (channel, is_thread) =
        if let Some(channel) = guild.channels.get(&channel_id.expect_channel()) {
            (channel, false)
        } else {
            let thread = guild
                .threads
                .iter()
                .find(|t| t.id == channel_id.expect_thread())
                .expect("Thread should exist if not found in channels.");

            let parent_channel = guild
                .channels
                .get(&thread.parent_id)
                .expect("Channel should be within the cache.");

            (parent_channel, true)
        };

    let mut permissions = guild.user_permissions_in(
        channel,
        guild
            .members
            .get(&ctx.cache.current_user().id)
            .expect("Bot member is always present in the guild cache."),
    );

    if is_thread && permissions.send_messages_in_threads() {
        permissions |= serenity::all::Permissions::SEND_MESSAGES;
    }

    Ok(permissions)
}

/// Extracts the message content while handling possible snapshot data.
fn extract_message_content(msg: &serenity::Message) -> Cow<'_, str> {
    if let Some(snapshot) = msg.message_snapshots.first() {
        Cow::Borrowed(snapshot.content.as_str())
    } else {
        Cow::Borrowed(msg.content.as_str())
    }
}

/// Handles OCR processing for image attachments ONCE and returns the extracted text.
async fn process_ocr_attachments(data: &Arc<crate::Data>, msg: &serenity::Message) -> Vec<String> {
    let mut extracted_texts = Vec::new();

    for attachment in &msg.attachments {
        if attachment.size >= 10_000_000 {
            continue; // Skip large files
        }

        if let Some(content_type) = &attachment.content_type {
            if matches!(
                content_type.as_str(),
                "image/jpeg" | "image/jpg" | "image/png" | "image/webp"
            ) {
                if let Ok(bytes) = attachment.download().await {
                    match data.ocr_engine.process(bytes).await {
                        Ok(string) => {
                            extracted_texts.push(string);
                        }
                        Err(e) => println!("Failed to process image with OCR engine: {e}"),
                    }
                }
            }
        }
    }

    extracted_texts
}

/// returns true for channel, otherwise false, second bool is if in category (only matters if first is true)
fn is_channel(
    ctx: &serenity::Context,
    msg: &serenity::Message,
) -> (bool, Option<serenity::ChannelId>) {
    let Some(guild) = ctx.cache.guild(msg.guild_id.unwrap()) else {
        return (true, None);
    };

    if let Some(channel) = guild
        .channels
        .iter()
        .find(|c| c.id == msg.channel_id.expect_channel())
    {
        return (true, channel.parent_id);
    }

    (false, None)
}

/* fn effective_channel(
    ctx: &serenity::Context,
    msg: &serenity::Message,
) -> Option<serenity::ChannelId> {
    let Some(guild) = ctx.cache.guild(msg.guild_id.unwrap()) else {
        return None
    };

    if let Some(channel) = guild.channels.iter().find(|c| c.id == msg.channel_id) {
        return channel.parent_id
    }

    if guild.threads.iter().find(|t| t.id == msg.channel_id).is_some() {
        Some(msg.channel_id)
    } else {
        None
    }
}
 */
