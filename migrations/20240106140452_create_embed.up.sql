CREATE TABLE embed (
    message_id BIGINT NOT NULL UNIQUE,
    channel_id BIGINT NOT NULL UNIQUE,
    server_id BIGINT NOT NULL UNIQUE
)