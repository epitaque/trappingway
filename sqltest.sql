-- attach 'c:\test\b.db3' as toMerge;           
-- BEGIN; 
-- insert into AuditRecords select * from toMerge.AuditRecords; 
-- COMMIT; 
-- detach toMerge;


-- DROP TABLE guilds;
-- DROP TABLE messages;

-- CREATE TABLE guilds (
--     guild_id TEXT PRIMARY KEY NOT NULL,
--     guild_name TEXT NOT NULL
-- );

-- CREATE TABLE messages (
--     message_id TEXT PRIMARY KEY NOT NULL,
--     channel_id TEXT NOT NULL,
--     data_center TEXT NOT NULL,
--     guild_id TEXT NOT NULL,
--     duty_name TEXT NOT NULL,
--   	FOREIGN KEY (guild_id) REFERENCES guilds (guild_id)
-- );

-- INSERT INTO guilds (guild_id, guild_name) VALUES (5, "UAR");
-- INSERT INTO guilds (guild_id, guild_name) VALUES (6, "UCR");
INSERT INTO messages (message_id, guild_id, datacenter, guild_id, duty_name, guild_id) VALUES (990278593877794877, 987311568452743188, "Chaos", 223932466912559105, "Dragonsong's Reprise (Ultimate)");
-- INSERT INTO messages (message_id, guild_id, datacenter) VALUES (1, 6, "Crystal");
-- INSERT INTO emotes (class_prefix, emote_uri, guild_id) VALUES ("SGE", "png", 5);
-- INSERT INTO emotes (class_prefix, emote_uri, guild_id) VALUES ("PLD", "png", 6);

-- SELECT * FROM guilds;
-- SELECT * FROM messages;
-- SELECT * FROM emotes;