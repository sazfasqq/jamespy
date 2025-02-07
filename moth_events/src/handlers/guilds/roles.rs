use crate::{helper::get_guild_name_override, Error};
use moth_ansi::{HI_GREEN, MAGENTA, RED, RESET};
use serenity::all::{Context, GuildId, Permissions, Role, RoleId};

use std::fmt::Write;

pub(crate) async fn role_create(ctx: &Context, role: &Role) -> Result<(), Error> {
    let guild_name = get_guild_name_override(ctx, &ctx.data(), Some(role.guild_id));

    // TODO: log details.

    println!(
        "{MAGENTA}[{guild_name}] A role called {} was created.",
        role.name
    );

    Ok(())
}

pub(crate) async fn role_delete(
    ctx: &Context,
    guild_id: GuildId,
    role_id: RoleId,
    role: Option<&Role>,
) -> Result<(), Error> {
    let guild_name = get_guild_name_override(ctx, &ctx.data(), Some(guild_id));

    if let Some(role) = role {
        println!(
            "{MAGENTA}[{guild_name}] A role called {} was deleted! (ID:{role_id})",
            role.name
        );
    } else {
        println!(
            "{MAGENTA}[{guild_name}] A role called {role_id} was deleted, but was not cached!"
        );
    }

    Ok(())
}

pub(crate) async fn role_update(
    ctx: &Context,
    old_role: Option<&Role>,
    role: &Role,
) -> Result<(), Error> {
    let guild_name = get_guild_name_override(ctx, &ctx.data(), Some(role.guild_id));

    let Some(old_role) = old_role else {
        println!(
            "{MAGENTA}[{guild_name}] {} (ID:{}) was updated but was not in the cache!",
            role.name, role.id
        );
        return Ok(());
    };

    let mut string = format!(
        "{MAGENTA}[{guild_name}] A role {} (ID:{}) was updated!",
        role.name, role.id
    );

    if old_role.name != role.name {
        writeln!(string, "name: {} -> {}", old_role.name, role.name).unwrap();
    }

    if old_role.colour != role.colour {
        let old = moth_ansi::from_colour(old_role.colour.0);
        let new = moth_ansi::from_colour(role.colour.0);

        // Basically, its only none if there was no colour, so basically don't print if no colour?
        // kinda because... theres no point
        if let (Some(old_col), Some(new_col)) = (old, new) {
            writeln!(
                string,
                "colour: #{old_col}{}{RESET} -> #{new_col}{}{RESET}",
                old_role.colour.0, role.colour.0
            )
            .unwrap();
        } else if let Some(new_col) = new {
            writeln!(string, "colour: None -> #{new_col}{}{RESET}", role.colour.0).unwrap();
        } else if let Some(old_col) = old {
            writeln!(
                string,
                "colour: #{old_col}{}{RESET} -> None",
                old_role.colour.0
            )
            .unwrap();
        }
    }

    if old_role.position != role.position {
        writeln!(
            string,
            "position: {} -> {}",
            old_role.position, role.position
        )
        .unwrap();
    }

    // TODO: write some stuff for the RoleTags even though it'll hardly ever change.

    if old_role.icon != role.icon {
        writeln!(string, "Icon has changed!").unwrap();
    }

    if old_role.unicode_emoji != role.unicode_emoji {
        let old = &old_role.unicode_emoji;
        let new = &role.unicode_emoji;

        if let (Some(old_emoji), Some(new_emoji)) = (old, new) {
            writeln!(string, "Emoji changed: {old_emoji} -> {new_emoji}").unwrap();
        } else if let Some(new_emoji) = new {
            writeln!(string, "emoji: None -> {new_emoji}").unwrap();
        } else if let Some(old_emoji) = old {
            writeln!(string, "old emoji: {old_emoji} -> None").unwrap();
        }
    }

    if old_role.permissions != role.permissions {
        let changes = permission_changes(old_role.permissions, role.permissions);
        writeln!(string, "{changes}").unwrap();
    };

    if old_role.hoist() != role.hoist() {
        writeln!(string, "hoisted: {} -> {}", old_role.hoist(), role.hoist()).unwrap();
    }

    if old_role.managed() != role.managed() {
        writeln!(
            string,
            "managed: {} -> {}",
            old_role.managed(),
            role.managed()
        )
        .unwrap();
    }

    if old_role.mentionable() != role.mentionable() {
        writeln!(
            string,
            "mentionable: {} -> {}",
            old_role.mentionable(),
            role.mentionable()
        )
        .unwrap();
    }

    println!("{string}");

    Ok(())
}

fn permission_changes(old: Permissions, new: Permissions) -> String {
    let added = new - old;
    let removed = old - new;

    let mut changes = String::new();

    for add in added {
        writeln!(
            changes,
            "{HI_GREEN}+ {}",
            add.get_permission_names().first().unwrap()
        )
        .unwrap();
    }

    for remove in removed {
        writeln!(
            changes,
            "{RED}- {}",
            remove.get_permission_names().first().unwrap()
        )
        .unwrap();
    }

    changes.pop();
    changes
}
