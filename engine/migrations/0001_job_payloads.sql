CREATE TABLE IF NOT EXISTS job_payloads (
    job_id UUID PRIMARY KEY,
    payload BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
