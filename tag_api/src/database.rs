use std::{fmt::Display, path::PathBuf, sync::OnceLock};

use actix_web::web::to;
use serde::{Deserialize, Serialize};

use crate::response::{ImageDbInfo};

pub trait Database {
    async fn get_image_location(
        &self,
        id: u32,
        auth_level: AuthLevel,
    ) -> Result<PathBuf, SqlDatabaseError>;

    async fn get_thumbnail_location(
        &self,
        id: u32,
        auth_level: AuthLevel,
    ) -> Result<PathBuf, SqlDatabaseError>;

    async fn get_filtered_images_paginated(
        &self,
        characters: Option<Vec<&str>>,
        tags: Option<Vec<&str>>,
        rating: Option<Rating>,
        per_page: u32,
        page: u32,
    ) -> Result<PaginatedResult<Image>, sqlx::error::Error>;

    async fn get_auth_level(&self, token: &str) -> Result<AuthLevel, SqlDatabaseError>;
    async fn get_filtered_tags_paginated(
        &self,
        tag: Option<&str>,
        per_page: u32,
        page: u32,
    ) -> Result<PaginatedResult<(String, u32)>, sqlx::error::Error>;
    async fn get_filtered_characters_paginated(
        &self,
        character: Option<&str>,
        per_page: u32,
        page: u32,
    ) -> Result<PaginatedResult<(String, u32)>, sqlx::error::Error>;
    async fn get_image_information(
        &self,
        id: u32,
        auth_level: AuthLevel,
    ) -> Result<ImageDbInfo, sqlx::error::Error>;
}

#[derive(Debug, Clone, Copy, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "rating")]
#[sqlx(rename_all = "PascalCase")]
pub enum AuthLevel {
    Guest,
    User,
    PrivilegedUser,
    Admin,
}

pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_items: u32,
    pub total_pages: u32,
}

impl AuthLevel {
    fn is_allowed(&self, rating: Rating) -> bool {
        match self {
            AuthLevel::Admin => true,
            AuthLevel::PrivilegedUser => true,
            _ => rating == Rating::General || rating == Rating::Sensitive,
        }
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type, PartialEq, Eq, Deserialize, Serialize)]
#[sqlx(type_name = "rating")]
#[sqlx(rename_all = "lowercase")]
pub enum Rating {
    General,
    Sensitive,
    Questionable,
    Explicit,
}

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct SqlDatabase {
    pool: sqlx::postgres::PgPool,
}

pub static IMAGE_PATH: OnceLock<PathBuf> = OnceLock::new();

impl SqlDatabase {
    pub async fn new(connection_string: &str) -> Result<Self, sqlx::error::Error> {
        let pool = sqlx::postgres::PgPool::connect(connection_string).await?;

        Ok(Self { pool })
    }
}

impl Database for SqlDatabase {
    async fn get_image_location(
        &self,
        id: u32,
        auth_level: AuthLevel,
    ) -> Result<PathBuf, SqlDatabaseError> {
        let record = sqlx::query!(
            "SELECT id, rating as \"rating:Rating\" FROM image WHERE id = $1 LIMIT 1",
            id as i32
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(SqlDatabaseError::SqlxError)?
        .ok_or(SqlDatabaseError::NotFound)?;

        if auth_level.is_allowed(record.rating) {
            let path = IMAGE_PATH.get().unwrap().join(format!("{}.png", record.id));

            Ok(path)
        } else {
            Err(SqlDatabaseError::NotAllowed)
        }
    }

    async fn get_thumbnail_location(
        &self,
        id: u32,
        auth_level: AuthLevel,
    ) -> Result<PathBuf, SqlDatabaseError> {
        let record = sqlx::query!(
            "SELECT id, rating as \"rating:Rating\" FROM image WHERE id = $1 AND thumbnail=true LIMIT 1",
            id as i32
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(SqlDatabaseError::SqlxError)?
        .ok_or(SqlDatabaseError::NotFound)?;

        if auth_level.is_allowed(record.rating) {
            let path = IMAGE_PATH
                .get()
                .unwrap()
                .join(format!("{}_thumbnail.jpg", record.id));

            Ok(path)
        } else {
            Err(SqlDatabaseError::NotAllowed)
        }
    }

    async fn get_filtered_images_paginated(
        &self,
        characters: Option<Vec<&str>>,
        tags: Option<Vec<&str>>,
        rating: Option<Rating>,
        per_page: u32,
        page: u32,
    ) -> Result<PaginatedResult<Image>, sqlx::error::Error> {
        let tag_slice = tags.as_deref();
        let character_slice = characters.as_deref();

        let images = sqlx::query_as(
            r#"
            SELECT i.*
            FROM image i
            LEFT JOIN tag_images ti ON ti.image_id = i.id
            LEFT JOIN tag t ON t.id = ti.tag_id
            LEFT JOIN character_images ci ON ci.image_id = i.id
            LEFT JOIN character c ON c.id = ci.character_id
            WHERE 
                ($1 IS NULL OR t.tag = ANY($1::text[]))
            AND
                ($2 IS NULL OR c.character = ANY($2::text[]))
            AND 
                ($3 IS NULL OR i.rating = $3)
            GROUP BY i.id
            HAVING
                ($1 IS NULL OR COUNT(DISTINCT t.tag) = cardinality($1))
            AND
                ($2 IS NULL OR COUNT(DISTINCT c.character) = cardinality($2))
            LIMIT $4
            OFFSET $5
            "#,
        )
        .bind(tag_slice)
        .bind(character_slice)
        .bind(rating)
        .bind(per_page as i32)
        .bind((page * per_page) as i32)
        .fetch_all(&self.pool)
        .await?;

        let (count,): (i64,) = sqlx::query_as(
            r#"
    SELECT COUNT(*) FROM (
        SELECT i.id
        FROM image i
        LEFT JOIN tag_images ti ON ti.image_id = i.id
        LEFT JOIN tag t ON t.id = ti.tag_id
        LEFT JOIN character_images ci ON ci.image_id = i.id
        LEFT JOIN character c ON c.id = ci.character_id
        WHERE 
            ($1 IS NULL OR t.tag = ANY($1::text[]))
        AND
            ($2 IS NULL OR c.character = ANY($2::text[]))
        AND 
            ($3 IS NULL OR i.rating = $3)
        GROUP BY i.id
        HAVING
            ($1 IS NULL OR COUNT(DISTINCT t.tag) = cardinality($1))
        AND
            ($2 IS NULL OR COUNT(DISTINCT c.character) = cardinality($2))
    ) filtered
    "#,
        )
        .bind(tag_slice)
        .bind(character_slice)
        .bind(rating)
        .fetch_one(&self.pool)
        .await?;
        let total_items: u32 = count as u32;

        Ok(PaginatedResult {
            items: images,
            total_items,
            total_pages: total_items.div_ceil(per_page),
        })
    }

    async fn get_filtered_tags_paginated(
        &self,
        tag: Option<&str>,
        per_page: u32,
        page: u32,
    ) -> Result<PaginatedResult<(String, u32)>, sqlx::error::Error> {
        let tag = tag.unwrap_or("");
        let like_pattern = format!("%{}%", tag);
        let tags = sqlx::query!(
            r#"
            SELECT tag, COUNT(*) as count
            FROM tag_images
            JOIN public.tag t on tag_images.tag_id = t.id
            WHERE t.tag LIKE $1
            GROUP BY t.tag
            ORDER BY COUNT(*) DESC
            LIMIT $2 
            OFFSET $3;
            "#,
            like_pattern,
            per_page as i64,
            (page * per_page) as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let tags = tags
            .iter()
            .map(|x| (x.tag.clone(), x.count.unwrap() as u32))
            .collect();

        let total_items = sqlx::query_scalar!(
            r#"
            SELECT count(*)
            FROM tag
            WHERE tag LIKE $1
            "#,
            like_pattern
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap() as u32;

        Ok(PaginatedResult {
            items: tags,
            total_items,
            total_pages: total_items.div_ceil(per_page),
        })
    }

    async fn get_auth_level(&self, token: &str) -> Result<AuthLevel, SqlDatabaseError> {
        let t = sqlx::query!(
            "SELECT level as \"level:AuthLevel\" FROM auth WHERE token = digest($1, 'sha256')",
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(SqlDatabaseError::SqlxError)?
        .ok_or(SqlDatabaseError::NotFound)?;

        Ok(t.level)
    }
    async fn get_filtered_characters_paginated(
        &self,
        character: Option<&str>,
        per_page: u32,
        page: u32,
    ) -> Result<PaginatedResult<(String, u32)>, sqlx::error::Error> {
        let character = character.unwrap_or("");
        let like_pattern = format!("%{}%", character);
        let tags = sqlx::query!(
            r#"
            SELECT character, COUNT(*) as count
            FROM character_images
            JOIN public.character t on character_images.character_id = t.id
            WHERE t.character LIKE $1
            GROUP BY t.character
            ORDER BY COUNT(*) DESC
            LIMIT $2 
            OFFSET $3;
            "#,
            like_pattern,
            per_page as i64,
            (page * per_page) as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let tags = tags
            .iter()
            .map(|x| (x.character.clone(), x.count.unwrap() as u32))
            .collect();

        let total_items = sqlx::query_scalar!(
            r#"
            SELECT count(*)
            FROM tag
            WHERE tag LIKE $1
            "#,
            like_pattern
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap() as u32;

        Ok(PaginatedResult {
            items: tags,
            total_items,
            total_pages: total_items.div_ceil(per_page),
        })
    }
    async fn get_image_information(
        &self,
        id: u32,
        auth_level: AuthLevel,
    ) -> Result<ImageDbInfo, sqlx::error::Error> {
        let tags: Vec<String> = sqlx::query_scalar!(
            r#"
            SELECT tag FROM tag
            JOIN public.tag_images ti on tag.id = ti.tag_id
            WHERE image_id = $1;
            "#,
            id as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let characters: Vec<String> = sqlx::query_scalar!(
            r#"
            SELECT character FROM character 
            JOIN public.character_images ci on character.id = ci.character_id 
            WHERE image_id = $1;
            "#,
            id as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let rating = sqlx::query_scalar!(
            "SELECT rating as \"rating:Rating\" FROM image WHERE id = $1;",
            id as i64
        )
        .fetch_one(&self.pool)
        .await?;

        let imageinfo = ImageDbInfo{
            id,
            tags,
            characters,
            rating
        };

        Ok(imageinfo)
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Image {
    pub id: i32,
}

#[derive(Debug)]
pub enum SqlDatabaseError {
    NotFound,
    SqlxError(sqlx::error::Error),
    NotAllowed,
}
