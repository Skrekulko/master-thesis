CREATE SCHEMA IF NOT EXISTS postgres;

CREATE TABLE IF NOT EXISTS postgres.CiphertextDistances (
    id INT PRIMARY KEY,
    distance INT
);

CREATE TABLE IF NOT EXISTS postgres.PlaintextDistances (
    id INT PRIMARY KEY,
    distance INT
);
