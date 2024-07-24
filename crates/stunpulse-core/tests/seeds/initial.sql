CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE tasks (
    task_id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    task_version TEXT NOT NULL,
    task_name TEXT NOT NULL,
    task_parameters JSONB,
    task_metadata JSONB
);
