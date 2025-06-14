-- Add down migration script here

DROP TABLE IF EXISTS "character_images";
DROP TABLE IF EXISTS "tag_images";
DROP TABLE IF EXISTS "character";
DROP TABLE IF EXISTS "tag";
DROP TABLE IF EXISTS "image";
DROP TYPE IF EXISTS "rating";
