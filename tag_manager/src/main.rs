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
async fn main() -> !{
    let _ = dotenv();

    let config = Config::create();
    set_static_vars(&config);

    let database = SqlDatabase::create(&config).await.unwrap();

    loop {
        process_images(&database).await.unwrap();
        sleep(Duration::new(120, 0)).await;
    }
}


fn set_static_vars(config : &Config){
    image_path::STORAGE_PATH.set(config.storage_path.clone()).unwrap();
    image_path::IMPORT_PATH.set(config.import_path.clone()).unwrap();
    image_path::VIDEO_PATH.set(config.video_path.clone()).unwrap();
    image_path::DISCARD_PATH.set(config.discarded_path.clone()).unwrap();
    tag_fetcher::TAGSERVICE_URL.set(config.tagmanager_url.clone()).unwrap();
}

#[derive(Clone, Debug)]
struct Config {
    connection_string: String,
    storage_path: PathBuf,
    import_path: PathBuf,
    discarded_path: PathBuf,
    video_path: PathBuf,
    tagmanager_url : String,
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
            video_path: PathBuf::from_str(
                env.get("VIDEO_DIR")
                    .map_or("/Images/Videos", |v| v),
            )
            .expect("Invalid other file type dir"),
            tagmanager_url: std::env::var("TAGSERVICE_URL").unwrap_or("http://127.0.0.1:8000".to_string()),
        }
    }
}
