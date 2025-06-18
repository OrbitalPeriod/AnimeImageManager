use std::str::FromStr;

use actix_web::{middleware::Logger, web, App, HttpServer};
use database::SqlDatabase;
use dotenv::dotenv;
use endpoints::{find_images, image, root};
mod database;
use anyhow::Result;
use env_logger::Env;
use log::info;

mod endpoints;
mod response;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenv();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let config = load_config().unwrap();
    load_statics(&config).unwrap();
    let db = get_db(&config).await.unwrap();
    let address = std::net::SocketAddr::new(
        std::net::IpAddr::from_str(&config.address).unwrap(),
        config.port,
    );

    info!("Starting api server on {}", address);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .service(root)
            .service(find_images)
            .service(image)
    })
    .bind(address)?
    .run()
    .await
}

#[derive(Clone, Debug)]
struct Config {
    address: String,
    port: u16,
    database_url: String,
    image_url_prefix: String,
    image_storage_path: String,
}

fn load_config() -> Result<Config> {
    Ok(Config {
        address: std::env::var("API_ADDRESS").unwrap_or("0.0.0.0".into()),
        port: std::env::var("API_PORT")
            .map(|x| x.parse().unwrap())
            .unwrap_or(8081),
        image_url_prefix: std::env::var("IMAGE_URL_PREFIX")
            .unwrap_or("http://127.0.0.1:8080/image".to_string()),
        image_storage_path: std::env::var("STORAGE_DIR").unwrap_or("/Images/Storage".to_string()),
        database_url: std::env::var("DATABASE_URL")?,
    })
}
fn load_statics(config: &Config) -> Result<()> {
    endpoints::IMAGE_URL_PREFIX
        .set(config.image_url_prefix.clone())
        .unwrap();
    database::IMAGE_PATH
        .set(config.image_storage_path.clone().into())
        .unwrap();

    Ok(())
}

async fn get_db(config: &Config) -> Result<SqlDatabase> {
    Ok(SqlDatabase::new(&config.database_url).await?)
}
