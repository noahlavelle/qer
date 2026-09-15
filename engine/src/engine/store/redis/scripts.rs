use std::sync::LazyLock;

use redisclient::Script;

/// Atomically pops the next job off a queue's jobs list and records a reservation
/// for it.
///
/// KEYS[1] - the queue's jobs list (`qer:queue:{queue_id}:jobs`)
/// KEYS[2] - the reservation hash to create (`qer:reservation:{reservation_id}`)
/// ARGV[1] - the reserving worker's id
///
/// Returns the popped job id, or a falsy reply if the list was empty.
pub(super) static RESERVE: LazyLock<Script> = LazyLock::new(|| {
    Script::new(
        r#"
        local job_id = redis.call('RPOP', KEYS[1])
        if not job_id then
            return false
        end

        redis.call('HSET', KEYS[2], 'job_id', job_id, 'worker_id', ARGV[1])
        return job_id
        "#,
    )
});

/// Atomically checks whether a reservation belongs to the given worker and, if so,
/// deletes it.
///
/// KEYS[1] - the reservation hash (`qer:reservation:{reservation_id}`)
/// ARGV[1] - the acking worker's id
///
/// Returns one of three shapes, decoded on the Rust side as a `Vec<String>`:
///   ["missing"]     - no reservation exists with this id
///   ["denied"]      - a reservation exists but belongs to a different worker
///   ["ok", job_id]  - the reservation belonged to this worker and was removed
pub(super) static ACK: LazyLock<Script> = LazyLock::new(|| {
    Script::new(
        r#"
        local job_id = redis.call('HGET', KEYS[1], 'job_id')
        if not job_id then
            return {'missing'}
        end

        local worker_id = redis.call('HGET', KEYS[1], 'worker_id')
        if worker_id ~= ARGV[1] then
            return {'denied'}
        end

        redis.call('DEL', KEYS[1])
        return {'ok', job_id}
        "#,
    )
});
