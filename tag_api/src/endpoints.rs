use std::{fmt::format, sync::OnceLock};

use actix_web::{
    get,
    http::StatusCode,
    web::{self},
};
use log::error;
use tokio::io::AsyncReadExt;

use crate::{
    database::{Database, SqlDatabase, SqlDatabaseError},
    requests::{FindCharacterQuery, FindImageRequest, FindTagQuery, ImageRequest},
    response::{ApiResponse, CharacterData, ImageInfo, Imagedata, PaginatedResponse, TagData},
};

pub static IMAGE_PREFIX: OnceLock<String> = OnceLock::new();
pub static MAX_PER_PAGE: u32 = 400;

#[get("/")]
async fn root(_: web::Data<SqlDatabase>) -> ApiResponse<&'static str, ()> {
    ApiResponse::new_success("Site up and working")
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

#[get("/thumbnail/{id}")]
async fn thumbnail(
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

    let path = match data.get_thumbnail_location(id, level).await {
        Ok(path) => path,
        Err(SqlDatabaseError::NotFound) => {
            return ApiResponse::new_bad_request(
                "Incorrect image id or no thumbnail yet processed",
            );
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

    ApiResponse::new_binary(StatusCode::OK, buffer, "image/jpg")
}

#[get("/search")]
async fn find_images(
    data: web::Data<SqlDatabase>,
    query: web::Query<FindImageRequest>,
) -> ApiResponse<PaginatedResponse<Imagedata>, &'static str> {
    let characters: Option<Vec<_>> = query
        .characters
        .as_ref()
        .and_then(|x| if x.is_empty() { None } else { Some(x) })
        .map(|x| x.split(',').collect());
    let tags: Option<Vec<_>> = query
        .tags
        .as_ref()
        .and_then(|x| if x.is_empty() { None } else { Some(x) })
        .map(|x| x.split(',').collect());
    let page = query.pages.page.unwrap_or(0);
    let per_page = query
        .pages
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
                    "{}/image/{}{}",
                    IMAGE_PREFIX.get().unwrap(),
                    x.id,
                    query
                        .token
                        .as_ref()
                        .map(|x| format!("?token={}", x))
                        .as_ref()
                        .map_or("", |v| v)
                ),
                format!(
                    "{}/thumbnail/{}{}",
                    IMAGE_PREFIX.get().unwrap(),
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

    ApiResponse::new_success(PaginatedResponse::new(
        ids,
        &format!(
            "/search?{}{}{}{}",
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
        page,
        per_page,
        paged_result.total_items,
    ))
}

#[get("/tag")]
pub async fn search_tags(
    data: web::Data<SqlDatabase>,
    query: web::Query<FindTagQuery>,
) -> ApiResponse<PaginatedResponse<TagData>, &'static str> {
    let page = query.pages.page.unwrap_or(0);
    let per_page = query
        .pages
        .per_page
        .map(|x| if x <= MAX_PER_PAGE { x } else { MAX_PER_PAGE })
        .unwrap_or(MAX_PER_PAGE);

    let tags = match data
        .get_filtered_tags_paginated(query.tag.as_deref(), per_page, page)
        .await
    {
        Ok(tags) => tags,
        Err(e) => {
            error!("Sqlx error: {}", e);
            return ApiResponse::new_internal_server_error("Pain");
        }
    };

    let items = tags
        .items
        .iter()
        .map(|(name, count)| TagData {
            name: name.to_string(),
            count: *count,
        })
        .collect();

    ApiResponse::new_success(PaginatedResponse::new(
        items,
        &format!(
            "/tag?{}",
            query
                .tag
                .as_ref()
                .map(|x| format!("&tag={}", x))
                .as_ref()
                .map_or("", |x| x)
        ),
        page,
        per_page,
        tags.total_items,
    ))
}

#[get("/character")]
pub async fn search_characters(
    data: web::Data<SqlDatabase>,
    query: web::Query<FindCharacterQuery>,
) -> ApiResponse<PaginatedResponse<CharacterData>, &'static str> {
    let page = query.pages.page.unwrap_or(0);
    let per_page = query
        .pages
        .per_page
        .map(|x| if x <= MAX_PER_PAGE { x } else { MAX_PER_PAGE })
        .unwrap_or(MAX_PER_PAGE);

    let tags = match data
        .get_filtered_characters_paginated(query.character.as_deref(), per_page, page)
        .await
    {
        Ok(tags) => tags,
        Err(e) => {
            error!("Sqlx error: {}", e);
            return ApiResponse::new_internal_server_error("Pain");
        }
    };

    let items = tags
        .items
        .iter()
        .map(|(name, count)| CharacterData {
            name: name.to_string(),
            count: *count,
        })
        .collect();

    ApiResponse::new_success(PaginatedResponse::new(
        items,
        &format!(
            "/character?{}",
            query
                .character
                .as_ref()
                .map(|x| format!("&tag={}", x))
                .as_ref()
                .map_or("", |x| x)
        ),
        page,
        per_page,
        tags.total_items,
    ))
}

#[get("/imageinfo/{id}")]
async fn imageinfo(
    data: web::Data<SqlDatabase>,
    id: web::Path<u32>,
    query: web::Query<ImageRequest>,
) -> ApiResponse<ImageInfo, &'static str> {
    let id = id.into_inner();
    let level = if let Some(token) = &query.token {
        match data.get_auth_level(token).await {
            Ok(level) => level,
            Err(SqlDatabaseError::NotFound) => crate::database::AuthLevel::Guest,
            Err(SqlDatabaseError::NotAllowed) => unreachable!(),
            Err(e) => {
                error!("Unable to get level, falling back to guest: {e:?}");
                crate::database::AuthLevel::Guest
            }
        }
    } else {
        crate::database::AuthLevel::Guest
    };

    let info = match data.get_image_information(id, level).await {
        Ok(info) => info,
        Err(e) => {
            error!("Unable to get db: {e:?}");
            return ApiResponse::new_internal_server_error("pain");
        }
    };

    let data = ImageInfo {
        tags: info.tags,
        characters: info.characters,
        rating: info.rating,
        image_url: format!(
            "{}/image/{}{}",
            IMAGE_PREFIX.get().unwrap(),
            info.id,
            query
                .token
                .as_ref()
                .map(|x| format!("?token={x}"))
                .as_ref()
                .map_or("", |v| v)
        ),
        tag_url: format!(
            "{}/image/{}{}",
            IMAGE_PREFIX.get().unwrap(),
            info.id,
            query
                .token
                .as_ref()
                .map(|x| format!("?token={x}"))
                .as_ref()
                .map_or("", |v| v)
        ),
    };

    ApiResponse::new_success(data)
}
