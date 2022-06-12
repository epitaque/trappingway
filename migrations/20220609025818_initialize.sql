-- Add migration script here
CREATE TABLE guilds (
    guild_id TEXT PRIMARY KEY,
    guild_name TEXT NOT NULL
);

CREATE TABLE messages (
    message_id TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL,
    datacenter TEXT NOT NULL,
    guild_id TEXT NOT NULL,
  	FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
);
