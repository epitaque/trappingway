-- Add migration script here
ALTER TABLE messages
ADD is_news INTEGER;
ALTER TABLE messages
ADD description_regex_filter TEXT;