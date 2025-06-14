-- Add up migration script here
DROP TABLE IF EXISTS "character";
DROP TABLE IF EXISTS "tags";
DROP TABLE IF EXISTS "image";

CREATE TYPE rating AS ENUM ('general', 'sensitive', 'questionable', 'explicit');

CREATE TABLE "image" (
  id serial NOT NULL PRIMARY KEY,
  rating rating NOT NULL,
  hash BYTEA NOT NULL
);

CREATE TABLE "character" (
  id integer NOT NULL,
  tag TEXT NOT NULL,
  CONSTRAINT fk_image FOREIGN KEY (id) REFERENCES image(id)
);

CREATE TABLE "tags" (
  id integer NOT NULL,
  tag TEXT NOT NULL,
  CONSTRAINT fk_image FOREIGN KEY (id) REFERENCES image(id)
);
