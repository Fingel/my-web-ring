-- Your SQL goes here
CREATE TABLE `sources` (
    `id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `s_type` INTEGER NOT NULL DEFAULT 1,
    `weight` INTEGER NOT NULL DEFAULT 10,
    `url` TEXT NOT NULL UNIQUE,
    `last_modified` TIMESTAMP NULL DEFAULT NULL,
    `etag` TEXT NULL DEFAULT NULL,
    `added` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
