use std::sync::OnceLock;

use actix_web::{get, http::{header, StatusCode}, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use crate::{database::{Database, SqlDatabase, SqlDatabaseError}, response::{ApiResponse, FindImageResponse, Imagedata}};

pub static IMAGE_URL_PREFIX: OnceLock<String> = OnceLock::new();

#[get("/")]
async fn root(_: web::Data<SqlDatabase>) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/image/{id}")]
async fn image(data: web::Data<SqlDatabase>, id: web::Path<u32>) -> impl Responder {
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
) -> ApiResponse<FindImageResponse, &'static str> {
    let characters: Option<Vec<_>> = query.characters.as_ref().map(|x| x.split(',').collect());
    let tags: Option<Vec<_>> = query.tags.as_ref().map(|x| x.split(',').collect());

    let ids = match data.get_filtered_images(characters, tags).await {
        Ok(ids) => ids,
        Err(e) => {
            println!("Error: {:?}", e);
            return ApiResponse::new_internal_server_error("Internal server error");
        }
    };

    let ids: Vec<_> = ids
        .iter()
        .map(|x| Imagedata::new(x.id, format!("{}/{}", IMAGE_URL_PREFIX.get().unwrap(), x.id)))
        .collect();

    return ApiResponse::new_success(ids);

}

