use anyhow::{Context, Result};
use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};

use crate::config::RedisConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub features: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskResult {
    pub features: Vec<usize>,
    pub mean_f1_macro: f64,
}

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub fn new(config: &RedisConfig) -> Result<Self> {
        let client = Client::open(config.connection_string())
            .context("Failed to create Redis client")?;
        Ok(Self { client })
    }
    
    pub fn get_connection(&self) -> Result<Connection> {
        self.client
            .get_connection()
            .context("Failed to get Redis connection")
    }
}

/// Push a batch of tasks to Redis queue
pub fn push_tasks(
    conn: &mut Connection,
    queue: &str,
    tasks: &[Task],
) -> Result<()> {
    for task in tasks {
        let payload = serde_json::to_string(task)
            .context("Failed to serialize task")?;
        let _: () = conn.rpush(queue, payload)
            .context("Failed to push task to Redis")?;
    }
    
    Ok(())
}

/// Read one task from queue with atomic move to processing queue
/// Returns (task, raw_data) or None if queue is empty
pub fn read_task(
    conn: &mut Connection,
    queue: &str,
    processing_queue: &str,
) -> Result<Option<(Task, String)>> {
    let result: Option<String> = redis::cmd("LMOVE")
        .arg(queue)
        .arg(processing_queue)
        .arg("LEFT")
        .arg("RIGHT")
        .query(conn)
        .context("Failed to read task from Redis")?;
    
    match result {
        Some(raw) => {
            let task: Task = serde_json::from_str(&raw)
                .context("Failed to deserialize task")?;
            Ok(Some((task, raw)))
        }
        None => Ok(None),
    }
}

/// Push result and acknowledge (remove from processing queue)
pub fn push_result_and_ack(
    conn: &mut Connection,
    result_queue: &str,
    processing_queue: &str,
    result: &TaskResult,
    raw_task: &str,
) -> Result<()> {
    let result_json = serde_json::to_string(result)
        .context("Failed to serialize result")?;
    
    let _: () = conn.rpush(result_queue, result_json)
        .context("Failed to push result")?;
    let _: () = conn.lrem(processing_queue, 1, raw_task)
        .context("Failed to ack task")?;
    
    Ok(())
}

/// Read multiple results from queue using Lua script for atomicity
pub fn read_results_batch(
    conn: &mut Connection,
    queue: &str,
    processing_queue: &str,
    batch_size: usize,
) -> Result<Vec<(TaskResult, String)>> {
    let script = redis::Script::new(r#"
        local res = {}
        local n = tonumber(ARGV[1])
        
        for i = 1, n do
            local item = redis.call("RPOPLPUSH", KEYS[1], KEYS[2])
            if not item then
                break
            end
            table.insert(res, item)
        end
        
        return res
    "#);
    
    let items: Vec<String> = script
        .key(queue)
        .key(processing_queue)
        .arg(batch_size)
        .invoke(conn)
        .context("Failed to read results batch")?;
    
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        let result: TaskResult = serde_json::from_str(&item)
            .context("Failed to deserialize result")?;
        results.push((result, item));
    }
    
    Ok(results)
}

/// Acknowledge processed results (remove from processing queue)
pub fn ack_results(
    conn: &mut Connection,
    processing_queue: &str,
    raw_items: &[String],
) -> Result<()> {
    for item in raw_items {
        let _: () = conn.lrem(processing_queue, 1, item)
            .context("Failed to ack result")?;
    }
    Ok(())
}

/// Get queue length
pub fn queue_len(conn: &mut Connection, queue: &str) -> Result<usize> {
    let len: usize = conn.llen(queue)
        .context("Failed to get queue length")?;
    Ok(len)
}
