use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "bf-gb")]
#[command(about = "Gradient Boosting Feature Selection in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Push feature combinations to Redis queue
    Push {
        /// Number of total features (e.g., 43)
        #[arg(short, long, default_value = "43")]
        features: usize,

        /// Number of features to select (e.g., 4)
        #[arg(short, long, default_value = "4")]
        select: usize,

        /// Redis host
        #[arg(long, default_value = "127.0.0.1")]
        redis_host: String,

        /// Redis port
        #[arg(long, default_value = "6379")]
        redis_port: u16,

        /// Redis password (optional)
        #[arg(long)]
        redis_password: Option<String>,

        /// Batch size for pushing
        #[arg(short, long, default_value = "10000")]
        batch_size: usize,

        /// Task queue name
        #[arg(long, default_value = "mylist")]
        queue: String,
    },

    /// Train models by consuming tasks from Redis
    Train {
        /// Path to training data CSV
        #[arg(short, long)]
        data: String,

        /// Redis host
        #[arg(long, default_value = "127.0.0.1")]
        redis_host: String,

        /// Redis port
        #[arg(long, default_value = "6379")]
        redis_port: u16,

        /// Redis password (optional)
        #[arg(long)]
        redis_password: Option<String>,

        /// Task queue name
        #[arg(long, default_value = "mylist")]
        queue: String,

        /// Result queue name
        #[arg(long, default_value = "results")]
        result_queue: String,

        /// Number of CV folds
        #[arg(short, long, default_value = "5")]
        folds: usize,

        /// Number of estimators in Gradient Boosting
        #[arg(long, default_value = "100")]
        n_estimators: usize,

        /// Learning rate for Gradient Boosting
        #[arg(long, default_value = "0.1")]
        learning_rate: f64,

        /// Maximum depth of trees in Gradient Boosting
        #[arg(long, default_value = "3")]
        max_depth: usize,

        /// Random seed
        #[arg(long, default_value = "0")]
        random_seed: u64,
    },

    /// Collect results from Redis to DuckDB
    Collect {
        /// Redis host
        #[arg(long, default_value = "127.0.0.1")]
        redis_host: String,

        /// Redis port
        #[arg(long, default_value = "6379")]
        redis_port: u16,

        /// Redis password (optional)
        #[arg(long)]
        redis_password: Option<String>,

        /// Result queue name
        #[arg(long, default_value = "results")]
        queue: String,

        /// DuckDB database path
        #[arg(short, long, default_value = "results.duckdb")]
        duckdb: String,

        /// Table name
        #[arg(long, default_value = "results")]
        table: String,

        /// Batch size for collecting
        #[arg(short, long, default_value = "5000")]
        batch_size: usize,
    },
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

impl RedisConfig {
    pub fn connection_string(&self) -> String {
        match &self.password {
            Some(pwd) => format!("redis://:{}@{}:{}/", pwd, self.host, self.port),
            None => format!("redis://{}:{}/", self.host, self.port),
        }
    }
}