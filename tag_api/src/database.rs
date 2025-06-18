use std::{path::PathBuf, sync::OnceLock};

pub trait Database {
    async fn get_image_location(&self, id: u32) -> Result<PathBuf, SqlDatabaseError>;

    async fn get_filtered_images(
        &self,
        characters: Option<Vec<&str>>,
        tags: Option<Vec<&str>>,
    ) -> Result<Vec<Image>, sqlx::error::Error>;
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
    async fn get_image_location(&self, id: u32) -> Result<PathBuf, SqlDatabaseError> {
        let test: i32 =
            sqlx::query_scalar!("SELECT id FROM image WHERE id = $1 LIMIT 1", id as i32)
                .fetch_optional(&self.pool)
                .await
                .map_err(SqlDatabaseError::SqlxError)?
                .ok_or(SqlDatabaseError::FileNotFound)?;

        let path = IMAGE_PATH.get().unwrap().join(format!("{}.png", test));

        Ok(path)
    }

    async fn get_filtered_images(
        &self,
        characters: Option<Vec<&str>>,
        tags: Option<Vec<&str>>,
    ) -> Result<Vec<Image>, sqlx::error::Error> {
        let tag_slice = tags.as_deref();
        let character_slice = characters.as_deref();

        sqlx::query_as(
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
            GROUP BY i.id
            HAVING
                ($1 IS NULL OR COUNT(DISTINCT t.tag) = cardinality($1))
            AND
                ($2 IS NULL OR COUNT(DISTINCT c.character) = cardinality($2));
            "#,
        )
        .bind(tag_slice)
        .bind(character_slice)
        .fetch_all(&self.pool)
        .await
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Image {
    pub id: i32,
}

#[derive(Debug)]
pub enum SqlDatabaseError {
    FileNotFound,
    SqlxError(sqlx::error::Error),
}
