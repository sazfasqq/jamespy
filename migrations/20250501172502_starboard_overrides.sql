CREATE TABLE starboard_overrides(
    channel_id BIGINT NOT NULL PRIMARY KEY,
    star_count SMALLINT NOT NULL,
    FOREIGN KEY (channel_id) REFERENCES channels(channel_id)
)
