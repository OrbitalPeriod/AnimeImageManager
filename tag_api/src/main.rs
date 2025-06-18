use std::{str::FromStr, sync::OnceLock};

use actix_web::{App, HttpResponse, HttpServer, Responder, get, http::header, web};
use database::{Database, SqlDatabase, SqlDatabaseError};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
mod database;
use anyhow::{Result};

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let _ = dotenv().ok();
    let config = load_config().unwrap();
    load_statics(&config).unwrap();
    let db = get_db(&config).await.unwrap();
    let address = std::net::SocketAddr::new(
        std::net::IpAddr::from_str(&config.address).unwrap(),
        config.port,
    );

    println!("Starting api server on {}", address);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(hello)
            .service(echo)
            .service(find_images)
    })
    .bind(address)?
    .run()
    .await
}

#[derive(Clone, Debug)]
struct Config {
    address: String,
    port: u16,
    database_url : String,
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
                .unwrap_or("https://127.0.0.1:8080/image".to_string()),
            image_storage_path: std::env::var("STORAGE_DIR").unwrap_or("/Images/Storage".to_string()),
            database_url: std::env::var("DATABASE_URL")?,
        })
}
fn load_statics(config: &Config) -> Result<()>{
    IMAGE_URL_PREFIX
        .set(config.image_url_prefix.clone())
        .unwrap();
    database::IMAGE_PATH.set(config.image_storage_path.clone().into()).unwrap();

    Ok(())
}

static IMAGE_URL_PREFIX: OnceLock<String> = OnceLock::new();

async fn get_db(config: &Config) -> Result<SqlDatabase> {
    Ok(SqlDatabase::new(&config.database_url).await?)
}

#[get("/")]
async fn hello(_: web::Data<SqlDatabase>) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/image/{id}")]
async fn echo(data: web::Data<SqlDatabase>, id: web::Path<u32>) -> impl Responder {
    let id = id.into_inner();

    let path = match data.get_image_location(id).await {
        Ok(path) => path,
        Err(SqlDatabaseError::FileNotFound) => {
            return HttpResponse::BadRequest().body("Incorrect image id");
        }
        Err(e) => {
            println!("sqlx error: {:?}", e);
            return HttpResponse::InternalServerError().body("Internal server error");
        }
    };

    let mut file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(e) => {
            println!("Error opening file: {:?}", e);
            return HttpResponse::InternalServerError().body("InternalServerError");
        }
    };

    let mut buffer = Vec::new();

    if let Err(e) = file.read_to_end(&mut buffer).await {
        println!("Error reading file: {:?}", e);
        return HttpResponse::InternalServerError().body("nooo");
    }

    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "image/png"))
        .body(buffer)
}

#[derive(Debug, Deserialize)]
struct FindImageRequest {
    characters: Option<String>,
    tags: Option<String>,
}

#[get("/images")]
async fn find_images(
    data: web::Data<SqlDatabase>,
    query: web::Query<FindImageRequest>,
) -> impl Responder {
    let characters: Option<Vec<_>> = query.characters.as_ref().map(|x| x.split(',').collect());
    let tags: Option<Vec<_>> = query.tags.as_ref().map(|x| x.split(',').collect());

    let ids = match data.get_filtered_images(characters, tags).await {
        Ok(ids) => ids,
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::InternalServerError().body("PAIN");
        }
    };

    let ids: Vec<_> = ids
        .iter()
        .map(|x| FindImageResponse {
            id: x.id,
            url: format!("{}/{}", IMAGE_URL_PREFIX.get().unwrap(), x.id),
        })
        .collect();

    HttpResponse::Ok().json(ids)
}

#[derive(Serialize)]
struct FindImageResponse {
    id: i32,
    url: String,
}
