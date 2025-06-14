use std::{collections::HashMap, env, path::PathBuf, str::FromStr};

use crate::database::Database;
mod database;
use database::SqlDatabase;
use dotenv::dotenv;
use processor::process_images;

mod image_path;
mod processor;
mod tag_fetcher;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::create();
    let database = SqlDatabase::create(&config).await.unwrap();

    process_images(database).await.unwrap();
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
            storage_path: PathBuf::from_str(env.get("STORAGE_DIR").expect("storage path required"))
                .expect("Invalid path"),
            import_path: PathBuf::from_str(env.get("IMPORT_DIR").expect("import dir required"))
                .expect("Invalid import path"),
            discarded_path: PathBuf::from_str(
                env.get("DISCARDED_DIR").expect("DISCARDED_DIR required"),
            )
            .expect("Invalid discarded dir"),
        }
    }
}
