CREATE TABLE IF NOT EXISTS words (
  id serial primary key,
  name varchar not null,
  reg varchar not null,
  owner varchar );

ALTER SEQUENCE words_id_seq RESTART;
UPDATE words SET id = DEFAULT;

CREATE TABLE IF NOT EXISTS tags (
  id serial primary key,
  name varchar unique not null,
  value varchar not null,
  owner varchar );

ALTER SEQUENCE tags_id_seq RESTART;
UPDATE tags SET id = DEFAULT;

CREATE EXTENSION IF NOT EXISTS fuzzystrmatch;
