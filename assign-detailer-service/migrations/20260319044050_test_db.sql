CREATE TABLE users (
    id UUID PRIMARY KEY,
    role TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TYPE availability_status AS ENUM ('ONLINE', 'OFFLINE', 'BUSY');

CREATE TABLE detailer_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    last_known_latitude DOUBLE PRECISION,
    last_known_longitude DOUBLE PRECISION,
    availability_status availability_status NOT NULL DEFAULT 'OFFLINE',
    rating DOUBLE PRECISION,
    total_jobs_completed INTEGER DEFAULT 0
);

CREATE TABLE orders (
    id UUID PRIMARY KEY,
    detailer_id UUID REFERENCES users(id),
    time_slot TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL
);