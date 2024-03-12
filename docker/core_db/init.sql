CREATE SCHEMA IF NOT EXISTS postgres;

CREATE TABLE IF NOT EXISTS postgres.CiphertextDistances (
    id BYTEA PRIMARY KEY,
    distance BYTEA
);

CREATE TABLE IF NOT EXISTS postgres.PlaintextDistances (
    id BYTEA PRIMARY KEY,
    distance BYTEA
);
