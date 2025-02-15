CREATE TABLE regexes (
    id SERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE,
    channel_id BIGINT,  -- NULL if global regex, otherwise set to specific channel
    pattern TEXT NOT NULL,
    recurse_channels BOOLEAN NOT NULL DEFAULT TRUE,
    recurse_threads BOOLEAN NOT NULL DEFAULT TRUE,
    -- bitflag
    detection_type SMALLINT NOT NULL DEFAULT 1
);

CREATE TABLE regex_exceptions (
    regex_id INT NOT NULL REFERENCES regexes(id) ON DELETE CASCADE,
    channel_id BIGINT NOT NULL,
    PRIMARY KEY (regex_id, channel_id)
);

CREATE TABLE responses (
    id SERIAL PRIMARY KEY,
    regex_id INT NOT NULL REFERENCES regexes(id) ON DELETE CASCADE,
    message TEXT,
    emote_id INT REFERENCES emotes(id) ON DELETE CASCADE
);
