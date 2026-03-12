use anyhow::{Context, Result};
use duckdb::{params, Connection};
use tracing::info;

use crate::redis_client::{RedisClient, TaskResult, ack_results, queue_len, read_results_batch};

pub struct Collector {
    redis_client: RedisClient,
    duckdb_path: String,
    table_name: String,
    batch_size: usize,
}

impl Collector {
    pub fn new(
        redis_client: RedisClient,
        duckdb_path: String,
        table_name: String,
        batch_size: usize,
    ) -> Self {
        Self {
            redis_client,
            duckdb_path,
            table_name,
            batch_size,
        }
    }
    
    pub fn run(&self, queue: &str) -> Result<()> {
        let processing_queue = format!("{}:processing", queue);
        
        // Connect to DuckDB
        let duck_conn = Connection::open(&self.duckdb_path)
            .context("Failed to open DuckDB connection")?;
        
        // Create table if not exists - use VARCHAR for features to store JSON array
        duck_conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    features VARCHAR,
                    mean_f1_macro DOUBLE
                )",
                self.table_name
            ),
            [],
        ).context("Failed to create table")?;
        
        let mut redis_conn = self.redis_client.get_connection()?;
        let mut total_count = 0usize;
        let mut batch_counter = 0usize;
        
        info!("Starting result collection from Redis to DuckDB...");
        
        // Begin transaction
        duck_conn.execute("BEGIN TRANSACTION;", [])
            .context("Failed to begin transaction")?;
        
        loop {
            // Read batch from Redis
            let results = read_results_batch(
                &mut redis_conn,
                queue,
                &processing_queue,
                self.batch_size,
            )?;
            
            if results.is_empty() {
                // Check if queues are really empty
                let remaining = queue_len(&mut redis_conn, queue)?;
                let processing = queue_len(&mut redis_conn, &processing_queue)?;
                
                if remaining == 0 && processing == 0 {
                    info!("All results collected. Total: {}", total_count);
                    break;
                }
                
                // Wait a bit for more results
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            
            let batch_count = results.len();
            let raw_items: Vec<String> = results.iter().map(|(_, raw)| raw.clone()).collect();
            
            // Insert into DuckDB
            let mut stmt = duck_conn.prepare(
                &format!("INSERT INTO {} VALUES (?, ?)", self.table_name)
            )?;
            
            for (result, _) in &results {
                // Store features as JSON string
                let features_json = serde_json::to_string(&result.features)
                    .unwrap_or_else(|_| format!("{:?}", result.features));
                stmt.execute(params![
                    features_json,
                    result.mean_f1_macro,
                ])?;
            }
            
            // Acknowledge results
            ack_results(&mut redis_conn, &processing_queue, &raw_items)?;
            
            total_count += batch_count;
            batch_counter += 1;
            
            // Commit every 10 batches
            if batch_counter % 10 == 0 {
                duck_conn.execute("COMMIT;", [])?;
                duck_conn.execute("BEGIN TRANSACTION;", [])?;
                info!("Collected {} results so far", total_count);
            }
        }
        
        // Final commit
        duck_conn.execute("COMMIT;", [])
            .context("Failed to commit final transaction")?;
        
        info!("Collection complete. Total results: {}", total_count);
        Ok(())
    }
    
    /// Query top N results
    pub fn query_top(&self, n: usize) -> Result<Vec<TaskResult>> {
        let conn = Connection::open(&self.duckdb_path)?;
        
        let mut stmt = conn.prepare(
            &format!(
                "SELECT features, mean_f1_macro FROM {} 
                 ORDER BY mean_f1_macro DESC LIMIT {}",
                self.table_name, n
            )
        )?;
        
        let rows = stmt.query_map([], |row| {
            let features_json: String = row.get(0)?;
            let mean_f1_macro: f64 = row.get(1)?;
            
            // Parse features from JSON string
            let features: Vec<usize> = serde_json::from_str(&features_json)
                .unwrap_or_default();
            
            Ok(TaskResult {
                features,
                mean_f1_macro,
            })
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        
        Ok(results)
    }
}
