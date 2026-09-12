CREATE TABLE IF NOT EXISTS job_records (
    job_id     UUID PRIMARY KEY,
    queue_name TEXT NOT NULL REFERENCES queues(name),
    status     TEXT NOT NULL CHECK (status IN ('ready', 'reserved')),
    worker_id  TEXT,
    enqueued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reserved_at TIMESTAMPTZ
);
