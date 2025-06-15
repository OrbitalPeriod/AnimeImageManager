use std::collections::HashMap;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, http::header, post, web};
use database::{Database, SqlDatabase, SqlDatabaseError};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenv().ok();
    let db = get_db().await.unwrap();

    println!("Starting api server");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(hello)
            .service(echo)
            .service(find_images)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}

static IMAGE_URL_PREFIX : &str = "http://127.0.0.1:8080/image";

async fn get_db() -> Result<SqlDatabase, Box<dyn std::error::Error>> {
    let map: HashMap<String, String> = HashMap::from_iter(std::env::vars());

    let connection_string = map.get("DATABASE_URL").unwrap();
    let storage_path = map.get("STORAGE_DIR").map_or("/Images/Storage", |v| v);

    Ok(SqlDatabase::new(connection_string, storage_path.into()).await?)
}

#[get("/")]
async fn hello(_: web::Data<SqlDatabase>) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/image/{id}")]
async fn echo(data: web::Data<SqlDatabase>, id: web::Path<u32>) -> impl Responder {
    let path = match data.get_image_location(id.into_inner()).await {
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
struct FindImageRequest{
    characters: Option<String>,
    tags: Option<String>,
}

#[get("/images")]
async fn find_images(data : web::Data<SqlDatabase>, query : web::Query<FindImageRequest>) ->impl Responder {
    let characters : Option<Vec<_>> = query.characters.as_ref().map(|x| x.split(',').collect());
    let tags : Option<Vec<_>> = query.tags.as_ref().map(|x| x.split(',').collect());

    let ids = match data.get_filtered_images(characters, dbg!(tags)).await{
        Ok(ids) => ids,
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::InternalServerError().body("PAIN");
        }
    };

    let ids : Vec<_> = ids.iter().map(|x| {
        FindImageResponse{
            id: x.id,
            url: format!("{}/{}", IMAGE_URL_PREFIX, x.id)
        }
    }).collect();

    HttpResponse::Ok().json(ids)
}

#[derive(Serialize)]
struct FindImageResponse{
    id: i32,
    url : String,
}
