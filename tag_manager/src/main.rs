use std::{collections::HashMap, env, path::PathBuf, str::FromStr, time::Duration};

use crate::database::Database;
mod database;
use database::SqlDatabase;
use dotenv::dotenv;
use processor::process_images;
use tokio::time::sleep;

mod image_path;
mod processor;
mod tag_fetcher;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::create();
    let database = SqlDatabase::create(&config).await.unwrap();

    loop {
        process_images(&database).await.unwrap();
        sleep(Duration::new(120, 0)).await;
    }
}

#[derive(Clone)]
struct Config {
    connection_string: String,
    storage_path: PathBuf,
    import_path: PathBuf,
    discarded_path: PathBuf,
}
impl Config {
    fn create() -> Config {
        let env: HashMap<String, String> = HashMap::from_iter(env::vars());

        Config {
            connection_string: env
                .get("DATABASE_URL")
                .expect("database connection string is required")
                .to_string(),
            storage_path: PathBuf::from_str(
                env.get("STORAGE_DIR").map_or("/Images/Storage", |v| v),
            )
            .expect("Invalid path"),
            import_path: PathBuf::from_str(env.get("IMPORT_DIR").map_or("/Images/Import", |v| v))
                .expect("Invalid import path"),
            discarded_path: PathBuf::from_str(
                env.get("DISCARDED_DIR").map_or("/Images/Discard", |v| v),
            )
            .expect("Invalid discarded dir"),
        }
    }
}
