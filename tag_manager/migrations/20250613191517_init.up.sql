-- Add up migration script here
DROP TABLE IF EXISTS "character_images";
DROP TABLE IF EXISTS "tag_images";
DROP TABLE IF EXISTS "character";
DROP TABLE IF EXISTS "tag";
DROP TABLE IF EXISTS "image";
DROP TYPE IF EXISTS "rating";

CREATE TYPE rating AS ENUM ('general', 'sensitive', 'questionable', 'explicit');

CREATE TABLE "image" (
  id serial NOT NULL PRIMARY KEY,
  rating rating NOT NULL,
  hash BYTEA NOT NULL
);

CREATE TABLE "character"(
  id serial NOT NULL UNIQUE,
  character TEXT NOT NULL PRIMARY KEY
);

CREATE TABLE "character_images" (
  image_id integer NOT NULL,
  character_id integer NOT NULL,
  CONSTRAINT fk_image FOREIGN KEY (image_id) REFERENCES image(id),
  CONSTRAINT fk_character FOREIGN KEY (character_id) REFERENCES character(id)
);

CREATE TABLE "tag"(
  id SERIAL NOT NULL UNIQUE,
  tag TEXT NOT NULL PRIMARY KEY
);

CREATE TABLE "tag_images" (
  image_id integer NOT NULL,
  tag_id integer NOT NULL,
  CONSTRAINT fk_image FOREIGN KEY (image_id) REFERENCES image(id),
  CONSTRAINT fk_tags FOREIGN KEY (tag_id) REFERENCES tag(id)
);
