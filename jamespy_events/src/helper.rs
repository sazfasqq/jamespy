use poise::serenity_prelude::{
    self as serenity, AutoArchiveDuration, ChannelId, ChannelType, Context, ForumLayoutType,
    GuildId, PermissionOverwriteType, Permissions, SortOrder,
};

// Helper function for getting the guild name even if ID is a None variant.
pub fn get_guild_name(ctx: &serenity::Context, guild_id: Option<GuildId>) -> String {
    if let Some(id) = guild_id {
        match id.name(ctx) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    } else {
        "None".to_string()
    }
}

// Helper function for getting the channel name.
pub async fn get_channel_name(
    ctx: &serenity::Context,
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
) -> String {
    match channel_id.name(ctx).await {
        Ok(name) => name,
        Err(_) => get_channel_name_thread(ctx, guild_id, channel_id).await,
    }
}

// Helper function for getting the channel name if its a thread.
async fn get_channel_name_thread(
    ctx: &serenity::Context,
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
) -> String {
    if guild_id.is_none() {
        return "Unknown Channel".to_string();
    }

    let id = guild_id.unwrap();
    let guild_cache = match ctx.cache.guild(id) {
        Some(cache) => cache,
        None => return "Unknown Channel".to_string(),
    };

    let threads = &guild_cache.threads;

    for thread in threads {
        if thread.id == channel_id.get() {
            return thread.name.to_string();
        }
    }

    "Unknown Channel".to_string()
}

pub fn channel_type_to_string(channel_type: ChannelType) -> String {
    match channel_type {
        ChannelType::Text => String::from("Text"),
        ChannelType::Private => String::from("Private"),
        ChannelType::Voice => String::from("Voice"),
        ChannelType::GroupDm => String::from("GroupDm"),
        ChannelType::Category => String::from("Category"),
        ChannelType::News => String::from("News"),
        ChannelType::NewsThread => String::from("NewsThread"),
        ChannelType::PublicThread => String::from("PublicThread"),
        ChannelType::PrivateThread => String::from("PrivateThread"),
        ChannelType::Stage => String::from("Stage"),
        ChannelType::Directory => String::from("Directory"),
        ChannelType::Forum => String::from("Forum"),
        ChannelType::Unknown(u) => format!("Unknown({u})"),
        _ => String::from("?"),
    }
}

pub fn overwrite_to_string(overwrite: PermissionOverwriteType) -> String {
    match overwrite {
        PermissionOverwriteType::Member(_) => String::from("Member"),
        PermissionOverwriteType::Role(_) => String::from("Role"),
        _ => String::from("?"),
    }
}

pub fn auto_archive_duration_to_string(duration: AutoArchiveDuration) -> String {
    match duration {
        AutoArchiveDuration::None => String::from("None"),
        AutoArchiveDuration::OneHour => String::from("1 hour"),
        AutoArchiveDuration::OneDay => String::from("1 day"),
        AutoArchiveDuration::ThreeDays => String::from("3 days"),
        AutoArchiveDuration::OneWeek => String::from("1 week"),
        AutoArchiveDuration::Unknown(u) => format!("Unknown({u})"),
        _ => String::from("?"),
    }
}

pub fn forum_layout_to_string(layout_type: ForumLayoutType) -> String {
    match layout_type {
        ForumLayoutType::NotSet => String::from("Not Set"),
        ForumLayoutType::ListView => String::from("List View"),
        ForumLayoutType::GalleryView => String::from("Gallery View"),
        ForumLayoutType::Unknown(u) => format!("Unknown({u})"),
        _ => String::from("?"),
    }
}

pub fn sort_order_to_string(sort_order: SortOrder) -> String {
    match sort_order {
        SortOrder::LatestActivity => String::from("Latest Activity"),
        SortOrder::CreationDate => String::from("Creation Date"),
        SortOrder::Unknown(u) => format!("Unknown({u})"),
        _ => String::from("?"),
    }
}

pub async fn get_permission_changes(
    ctx: Context,
    guild_id: GuildId,
    old_allow: Permissions,
    new_allow: Permissions,
    old_deny: Permissions,
    new_deny: Permissions,
    kind: PermissionOverwriteType,
) -> String {
    let name = match kind {
        PermissionOverwriteType::Member(user_id) => match user_id.to_user(ctx).await {
            Ok(user) => user.name.to_string(),
            Err(_) => String::from("Unknown User"),
        },
        PermissionOverwriteType::Role(role_id) => ctx
            .cache
            .guild(guild_id)
            .unwrap()
            .roles
            .get(&role_id)
            .map_or_else(|| "Unknown Role".to_string(), |role| role.name.to_string()),
        _ => String::from("Unknown"),
    };

    let mut changes_str = String::new();
    let kind_string = overwrite_to_string(kind);
    if old_allow != new_allow || old_deny != new_deny {
        changes_str.push_str(&format!(
            "Permission override for {name} ({kind_string}) changed!\n"
        ));
        let allow_changes_detail = get_permission_changes_detail(old_allow, new_allow, true);
        let deny_changes_detail = get_permission_changes_detail(old_deny, new_deny, false);

        if !allow_changes_detail.is_empty() {
            changes_str.push_str("allow:\n");
            changes_str.push_str(&allow_changes_detail);
        }

        if !deny_changes_detail.is_empty() {
            changes_str.push_str("deny:\n");
            changes_str.push_str(&deny_changes_detail);
        }
    }

    changes_str
}

pub fn get_permission_changes_detail(old: Permissions, new: Permissions, allow: bool) -> String {
    let mut changes_str = String::new();
    let added_color = if allow { "\x1B[92m" } else { "\x1B[31m" };
    let removed_color = if allow { "\x1B[31m" } else { "\x1B[92m" };

    let added_perms: Vec<String> = {
        let mut added = Vec::new();
        for permission in Permissions::all().iter() {
            let permission_name = format!("{permission}");
            if new.contains(permission) && !old.contains(permission) {
                added.push(permission_name);
            }
        }
        added
    };

    let removed_perms: Vec<String> = {
        let mut removed = Vec::new();
        for permission in Permissions::all().iter() {
            let permission_name = format!("{permission}");
            if !new.contains(permission) && old.contains(permission) {
                removed.push(permission_name);
            }
        }
        removed
    };

    if !added_perms.is_empty() {
        for perm in &added_perms {
            changes_str.push_str(&format!("{added_color}+ {perm}\n\x1B[0m"));
        }
    }

    if !removed_perms.is_empty() {
        for perm in &removed_perms {
            changes_str.push_str(&format!("{removed_color}- {perm}\n\x1B[0m"));
        }
    }

    changes_str
}