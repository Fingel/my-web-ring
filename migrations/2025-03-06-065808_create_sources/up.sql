-- Your SQL goes here
CREATE TABLE `sources` (
    `id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `url` TEXT NOT NULL,
    `added` TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
