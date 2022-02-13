CREATE TABLE IF NOT EXISTS words (
  id serial primary key,
  name varchar not null,
  reg varchar not null,
  owner varchar );

CREATE TABLE IF NOT EXISTS tags (
  id serial primary key,
  name varchar unique not null,
  value varchar not null,
  owner varchar );

CREATE EXTENSION IF NOT EXISTS fuzzystrmatch;
