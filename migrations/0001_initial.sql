-- Volunteers
CREATE TABLE IF NOT EXISTS volunteers (
    volunteer_id        TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    phone               TEXT NOT NULL,
    skills              TEXT[] NOT NULL DEFAULT '{}',
    area                TEXT NOT NULL,
    availability        TEXT NOT NULL DEFAULT 'available',
    last_lat            DOUBLE PRECISION,
    last_lng            DOUBLE PRECISION,
    location_updated_at TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tasks
CREATE TABLE IF NOT EXISTS tasks (
    task_id             TEXT PRIMARY KEY,
    incident_id         TEXT NOT NULL,
    title               TEXT NOT NULL,
    required_skills     TEXT[] NOT NULL DEFAULT '{}',
    location_id         TEXT NOT NULL,
    volunteers_needed   BIGINT NOT NULL,
    volunteers_matched  BIGINT NOT NULL DEFAULT 0,
    urgency             TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'open',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Matches — unique constraint enforces idempotency
CREATE TABLE IF NOT EXISTS matches (
    match_id        TEXT PRIMARY KEY,
    task_id         TEXT NOT NULL REFERENCES tasks(task_id),
    volunteer_id    TEXT NOT NULL REFERENCES volunteers(volunteer_id),
    status          TEXT NOT NULL DEFAULT 'pending',
    matched_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (task_id, volunteer_id)
);
