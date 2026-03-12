use anyhow::Result;
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

mod collector;
mod config;
mod generator;
mod gradient_boosting;
mod redis_client;
mod trainer;

use collector::Collector;
use config::{Cli, Commands, RedisConfig};
use generator::{BatchGenerator, combination_count};
use redis_client::{push_tasks, read_task, push_result_and_ack, RedisClient, Task};
use trainer::{Dataset, Trainer};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Push {
            features,
            select,
            redis_host,
            redis_port,
            redis_password,
            batch_size,
            queue,
        } => {
            cmd_push(
                features,
                select,
                &redis_host,
                redis_port,
                redis_password,
                batch_size,
                &queue,
            )?;
        }

        Commands::Train {
            data,
            redis_host,
            redis_port,
            redis_password,
            queue,
            result_queue,
            folds,
            n_estimators,
            learning_rate,
            max_depth,
            random_seed,
        } => {
            cmd_train(
                &data,
                &redis_host,
                redis_port,
                redis_password,
                &queue,
                &result_queue,
                folds,
                n_estimators,
                learning_rate,
                max_depth,
                random_seed,
            )?;
        }

        Commands::Collect {
            redis_host,
            redis_port,
            redis_password,
            queue,
            duckdb,
            table,
            batch_size,
        } => {
            cmd_collect(
                &redis_host,
                redis_port,
                redis_password,
                &queue,
                &duckdb,
                &table,
                batch_size,
            )?;
        }
    }

    Ok(())
}

fn cmd_push(
    n_features: usize,
    k_select: usize,
    redis_host: &str,
    redis_port: u16,
    redis_password: Option<String>,
    batch_size: usize,
    queue: &str,
) -> Result<()> {
    info!("Starting push command: C({}, {}) combinations", n_features, k_select);

    let total = combination_count(n_features, k_select);
    info!("Total combinations to push: {}", total);

    let redis_config = RedisConfig {
        host: redis_host.to_string(),
        port: redis_port,
        password: redis_password,
    };

    let redis_client = RedisClient::new(&redis_config)?;
    let mut conn = redis_client.get_connection()?;

    let mut generator = BatchGenerator::new(n_features, k_select, batch_size);
    let mut batch_num = 0usize;

    while let Some(batch) = generator.next_batch() {
        batch_num += 1;

        let tasks: Vec<Task> = batch
            .into_iter()
            .map(|features| Task { features })
            .collect();

        push_tasks(&mut conn, queue, &tasks)?;

        if batch_num % 10 == 0 || batch_num == 1 {
            info!("Pushed batch {} ({}/{})", batch_num, batch_num * batch_size, total);
        }
    }

    info!("Push complete! Total batches: {}", batch_num);
    Ok(())
}

fn cmd_train(
    data_path: &str,
    redis_host: &str,
    redis_port: u16,
    redis_password: Option<String>,
    queue: &str,
    result_queue: &str,
    n_folds: usize,
    n_estimators: usize,
    learning_rate: f64,
    max_depth: usize,
    random_seed: u64,
) -> Result<()> {
    info!("Starting train command");

    info!("Using GradientBoosting with:");
    info!("  n_estimators = {}", n_estimators);
    info!("  learning_rate = {}", learning_rate);
    info!("  max_depth = {}", max_depth);
    info!("  random_seed = {}", random_seed);

    let dataset = Dataset::from_csv(data_path)?;
    let trainer = Trainer::new(dataset, n_estimators, learning_rate, max_depth, n_folds, random_seed);

    let redis_config = RedisConfig {
        host: redis_host.to_string(),
        port: redis_port,
        password: redis_password,
    };

    let redis_client = RedisClient::new(&redis_config)?;
    let mut conn = redis_client.get_connection()?;

    let processing_queue = format!("{}:processing", queue);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("Received shutdown signal, finishing current task...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut task_count = 0usize;

    while running.load(Ordering::SeqCst) {
        let result = read_task(&mut conn, queue, &processing_queue)?;

        match result {
            Some((task, raw_task)) => {
                task_count += 1;

                match trainer.train_and_evaluate(&task) {
                    Ok(result) => {
                        if let Err(e) = push_result_and_ack(
                            &mut conn,
                            result_queue,
                            &processing_queue,
                            &result,
                            &raw_task,
                        ) {
                            warn!("Failed to push result: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Training failed for features {:?}: {}", task.features, e);
                    }
                }

                if task_count % 100 == 0 {
                    info!("Processed {} tasks", task_count);
                }
            }
            None => {
                info!("No more tasks in queue. Processed {} tasks total.", task_count);
                break;
            }
        }
    }

    Ok(())
}

fn cmd_collect(
    redis_host: &str,
    redis_port: u16,
    redis_password: Option<String>,
    queue: &str,
    duckdb_path: &str,
    table_name: &str,
    batch_size: usize,
) -> Result<()> {
    info!("Starting collect command");

    let redis_config = RedisConfig {
        host: redis_host.to_string(),
        port: redis_port,
        password: redis_password,
    };

    let redis_client = RedisClient::new(&redis_config)?;
    let collector = Collector::new(
        redis_client,
        duckdb_path.to_string(),
        table_name.to_string(),
        batch_size,
    );

    collector.run(queue)?;

    info!("Top 10 results:");
    let top = collector.query_top(10)?;
    for (i, result) in top.iter().enumerate() {
        info!(
            "  {}. Features {:?} -> F1-macro: {:.2}",
            i + 1,
            result.features,
            result.mean_f1_macro
        );
    }

    Ok(())
}