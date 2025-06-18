use std::sync::OnceLock;

use actix_web::{HttpResponse, Responder, get, http::header, web};
use log::error;
use serde::Deserialize;
use sqlx::query;
use tokio::io::AsyncReadExt;

use crate::{
    database::{Database, Rating, SqlDatabase, SqlDatabaseError},
    response::{ApiResponse, FindImageResponse, Imagedata},
};

pub static IMAGE_URL_PREFIX: OnceLock<String> = OnceLock::new();

#[get("/")]
async fn root(_: web::Data<SqlDatabase>) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug, Deserialize)]
struct ImageRequest {
    token: Option<String>,
}

#[get("/image/{id}")]
async fn image(
    data: web::Data<SqlDatabase>,
    id: web::Path<u32>,
    query: web::Query<ImageRequest>,
) -> impl Responder {
    let id = id.into_inner();
    let level = if let Some(token) = &query.token {
        match data.get_auth_level(token).await {
            Ok(level) => level,
            Err(SqlDatabaseError::NotFound) => crate::database::AuthLevel::Guest,
            Err(SqlDatabaseError::NotAllowed) => unreachable!(),
            Err(e) => {
                error!("Unable to get level, falling back to guest: {:?}", e);
                crate::database::AuthLevel::Guest
            }
        }
    } else {
        crate::database::AuthLevel::Guest
    };

    let path = match data.get_image_location(id, level).await {
        Ok(path) => path,
        Err(SqlDatabaseError::NotFound) => {
            return HttpResponse::BadRequest().body("Incorrect image id");
        }
        Err(SqlDatabaseError::NotAllowed) => {
            return HttpResponse::MethodNotAllowed().body("Not correct permissions for this image");
        }
        Err(e) => {
            error!("sqlx error: {:?}", e);
            return HttpResponse::InternalServerError().body("Internal server error");
        }
    };

    let mut file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(e) => {
            error!("Error opening file: {:?}", e);
            return HttpResponse::InternalServerError().body("InternalServerError");
        }
    };

    let mut buffer = Vec::new();

    if let Err(e) = file.read_to_end(&mut buffer).await {
        error!("Error reading file: {:?}", e);
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
    rating: Option<Rating>
}

#[get("/images")]
async fn find_images(
    data: web::Data<SqlDatabase>,
    query: web::Query<FindImageRequest>,
) -> ApiResponse<FindImageResponse, &'static str> {
    let characters: Option<Vec<_>> = query.characters.as_ref().map(|x| x.split(',').collect());
    let tags: Option<Vec<_>> = query.tags.as_ref().map(|x| x.split(',').collect());

    let ids = match data.get_filtered_images(characters, tags, query.rating).await {
        Ok(ids) => ids,
        Err(e) => {
            error!("Error: {:?}", e);
            return ApiResponse::new_internal_server_error("Internal server error");
        }
    };

    let ids: Vec<_> = ids
        .iter()
        .map(|x| {
            Imagedata::new(
                x.id,
                format!("{}/{}", IMAGE_URL_PREFIX.get().unwrap(), x.id),
            )
        })
        .collect();

    ApiResponse::new_success(ids)
}
