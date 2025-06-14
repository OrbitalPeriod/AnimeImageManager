use std::{clone, collections::HashMap, env, path::PathBuf, str::FromStr};

use crate::database::Database;
mod database;
use database::SqlDatabase;
use dotenv::dotenv;
use processor::process_images;

mod processor;
mod image_path;
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

fn test() {
    // Load both images
    let img1 = image::open("TestImages/laffey.jpeg").unwrap();
    let img2 = image::open("TestImages/2.png").unwrap();

    // Generate hashes
    let hash1 = imagehash::average_hash(&img1);
    let hash2 = imagehash::average_hash(&img2);
    // Compare hashes using Hamming distance

    println!("Hash 1: {}", hash1);
    println!("Hash 2: {}", hash2);

    println!("hash1 : {:?}", hash1.to_bytes());
}
