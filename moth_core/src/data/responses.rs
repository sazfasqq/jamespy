use std::collections::{HashMap, HashSet};

use dashmap::DashMap;
use regex::Regex;
use serenity::all::{GenericChannelId, GuildId, ReactionType};

#[derive(Default)]
pub struct ResponseCache {
    pub guild: DashMap<GuildId, GuildCache>,
}

#[derive(Default, Debug, Clone)]
pub struct GuildCache {
    pub global: Vec<RegexData>,
    pub channel: HashMap<GenericChannelId, Vec<RegexData>>,
}

/// Generic `RegexData` that fits over the `guild_regexes` and `channel_regexes`.
#[derive(Clone, Debug)]
pub struct RegexData {
    pub id: i32,
    /// Wether a category regex should go through the channels.
    pub pattern: Regex,
    pub recurse_channels: bool,
    /// Wether or not a global/channel regex goes all the way to threads.
    pub recurse_threads: bool,
    /// What response to send.
    pub response: ResponseType,
    /// What channels/threads to ignore.
    pub exceptions: HashSet<GenericChannelId>,
    pub detection_type: DetectionType,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DetectionType: u8 {
        const CONTENT = 0b00000001;
        const OCR     = 0b00000010;
    }
}

impl From<u8> for DetectionType {
    fn from(value: u8) -> Self {
        DetectionType::from_bits(value).expect("Invalid value for DetectionType")
    }
}

#[derive(Clone, Debug)]
pub enum ResponseType {
    Message(String),
    Emoji(ReactionType),
}

/* impl ResponseCache {
    pub(crate) fn get_guild_ref(
        &self,
        guild_id: GuildId,
    ) -> Option<dashmap::mapref::one::Ref<'_, GuildId, GuildCache>> {
        self.guild.get(&guild_id)
    }

    pub(crate) fn remove_guild(&self, guild_id: GuildId) {
        self.guild.remove(&guild_id);
    }

    pub(crate) fn remove_regex(&self, guild_id: GuildId, channel_id: Option<ChannelId>, id: i32) {
        if let Some(channel_id) = channel_id {
            self.guild.get_mut(&guild_id).map(|mut g| {
                g.channel
                    .get_mut(&channel_id)
                    .map(|c| c.retain(|i| i.id != id))
            });
        } else if let Some(mut g) = self.guild.get_mut(&guild_id) {
            g.global.retain(|i| i.id != id);
        }
    }

    pub(crate) fn add_regex(
        &self,
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        regex: RegexData,
    ) {
        if let Some(channel_id) = channel_id {
            if let Some(mut g) = self.guild.get_mut(&guild_id) {
                g.channel
                    .entry(channel_id)
                    .or_insert_with(Vec::new)
                    .push(regex);
            }
        } else if let Some(mut g) = self.guild.get_mut(&guild_id) {
            g.global.push(regex);
        }
    }

    pub(crate) fn replace_regex(
        &self,
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        id: i32,
        new_regex: RegexData,
    ) {
        if let Some(channel_id) = channel_id {
            if let Some(mut g) = self.guild.get_mut(&guild_id) {
                if let Some(c) = g.channel.get_mut(&channel_id) {
                    if let Some(existing) = c.iter_mut().find(|i| i.id == id) {
                        *existing = new_regex;
                    }
                }
            };
        } else if let Some(mut g) = self.guild.get_mut(&guild_id) {
            if let Some(existing) = g.global.iter_mut().find(|i| i.id == id) {
                *existing = new_regex;
            }
        }
    }
} */
