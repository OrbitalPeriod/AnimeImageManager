use std::{fmt::format, sync::OnceLock};

use actix_web::{
    get,
    http::{StatusCode, header},
    web,
};
use log::error;
use serde::Deserialize;
use tokio::io::AsyncReadExt;

use crate::{
    database::{Database, Rating, SqlDatabase, SqlDatabaseError},
    response::{ApiResponse, FindImageResponse, Imagedata},
};

pub static IMAGE_URL_PREFIX: OnceLock<String> = OnceLock::new();
pub static MAX_PER_PAGE: u32 = 400;

#[get("/")]
async fn root(_: web::Data<SqlDatabase>) -> ApiResponse<&'static str, ()> {
    ApiResponse::new_success("Site up and working")
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
) -> ApiResponse<(), &'static str> {
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
            return ApiResponse::new_bad_request("Incorrect image id");
        }
        Err(SqlDatabaseError::NotAllowed) => {
            return ApiResponse::new_not_allowed("Not correct permissions for this image");
        }
        Err(e) => {
            error!("sqlx error: {:?}", e);
            return ApiResponse::new_internal_server_error("Internal server error");
        }
    };

    let mut file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(e) => {
            error!("Error opening file: {:?}", e);
            return ApiResponse::new_internal_server_error("Internal server error");
        }
    };

    let mut buffer = Vec::new();

    if let Err(e) = file.read_to_end(&mut buffer).await {
        error!("Error reading file: {:?}", e);
        return ApiResponse::new_internal_server_error("Internal server error");
    }

    ApiResponse::new_binary(StatusCode::OK, buffer, "image/png")
}

#[derive(Debug, Deserialize)]
struct FindImageRequest {
    characters: Option<String>,
    tags: Option<String>,
    rating: Option<Rating>,
    page: Option<i32>,
    per_page: Option<u32>,
    token: Option<String>,
}

#[get("/search")]
async fn find_images(
    data: web::Data<SqlDatabase>,
    query: web::Query<FindImageRequest>,
) -> ApiResponse<FindImageResponse, &'static str> {
    let characters: Option<Vec<_>> = query.characters.as_ref().map(|x| x.split(',').collect());
    let tags: Option<Vec<_>> = query.tags.as_ref().map(|x| x.split(',').collect());
    let page = query.page.unwrap_or(0) as u32;
    let per_page = query
        .per_page
        .map(|x| if x <= MAX_PER_PAGE { x } else { MAX_PER_PAGE })
        .unwrap_or(MAX_PER_PAGE);

    let paged_result = match data
        .get_filtered_images_paginated(characters, tags, query.rating, per_page, page)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            error!("Error: {:?}", e);
            return ApiResponse::new_internal_server_error("Internal server error");
        }
    };

    let ids: Vec<_> = paged_result
        .items
        .iter()
        .map(|x| {
            Imagedata::new(
                x.id,
                format!(
                    "{}/{}{}",
                    IMAGE_URL_PREFIX.get().unwrap(),
                    x.id,
                    query
                        .token
                        .as_ref()
                        .map(|x| format!("?token={}", x))
                        .as_ref()
                        .map_or("", |v| v)
                ),
            )
        })
        .collect();

    ApiResponse::new_success(FindImageResponse {
        page,
        per_page,
        total_items: paged_result.total_items,
        total_pages: paged_result.total_items.div_ceil(per_page),
        items: ids,
        next: format!(
            "/search?page={}&per_page={}{}{}{}{}",
            page + 1,
            per_page,
            query
                .characters
                .as_ref()
                .map(|x| format!("&characters={}", x))
                .as_ref()
                .map_or("", |v| v),
            query
                .tags
                .as_ref()
                .map(|x| format!("&tags={}", x))
                .as_ref()
                .map_or("", |v| v),
            query
                .rating
                .map(|x| format!("&rating={}", x))
                .as_ref()
                .map_or("", |v| v),
            query
                .token
                .as_ref()
                .map(|x| format!("&token={}", x))
                .as_ref()
                .map_or("", |v| v)
        ),
        prev: format!(
            "/search?page={}&per_page={}{}{}{}{}",
            page.saturating_sub(1),
            per_page,
            query
                .characters
                .as_ref()
                .map(|x| format!("&characters={}", x))
                .as_ref()
                .map_or("", |v| v),
            query
                .tags
                .as_ref()
                .map(|x| format!("&tags={}", x))
                .as_ref()
                .map_or("", |v| v),
            query
                .rating
                .map(|x| format!("&rating={}", x))
                .as_ref()
                .map_or("", |v| v),
            query
                .token
                .as_ref()
                .map(|x| format!("&token={}", x))
                .as_ref()
                .map_or("", |v| v)
        ),
    })
}
