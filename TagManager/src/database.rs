use image::DynamicImage;
use sqlx::Error;

use imagehash::Hash;

use crate::{tag_fetcher::{Rating, Tags}, Config};

pub trait Database {
    async fn create(config: &Config) -> Result<impl Database, Error>;
    async fn save_image(&self, hash: &[u8;8], tags : &Tags) -> Result<u32, Error>;
    async fn check_hash(&self, hash: &[u8;8]) -> Result<bool, Error>;
    fn config(&self) -> &Config;
}

pub struct SqlDatabase {
    pool: sqlx::postgres::PgPool,
    config: Config,
}

impl Database for SqlDatabase {
    async fn create(config: &Config) -> Result<impl Database, Error> {
        Ok(Self {
            pool: sqlx::postgres::PgPool::connect(&config.connection_string).await?,
            config: config.clone(),
        })
    }

    async fn check_hash(&self, hash: &[u8;8]) -> Result<bool, Error> {
        let query: (bool,) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM image WHERE hash=$1)")
            .bind(hash)
            .fetch_one(&self.pool)
            .await?;

        Ok(query.0)
    }
    async fn save_image(&self, hash: &[u8;8], tags: &Tags) -> Result<u32, Error>{
        let rec : (i32, ) = sqlx::query_as("INSERT INTO image (rating, hash) VALUES ($1, $2) RETURNING id")
            .bind(tags.rating.clone() as Rating)
            .bind(hash)
            .fetch_one(&self.pool)
            .await?;

        let id = rec.0;

        if let Some(character_tags) = &tags.character_tags{
            for tag in character_tags{
                sqlx::query("INSERT INTO character (id, tag) VALUES ($1, $2)")
                .bind(id)
                .bind(tag)
                .execute(&self.pool)
                .await?;
            }
        }
        if let Some(character_tags) = &tags.general_tags{
            for tag in character_tags{
                sqlx::query("INSERT INTO tags (id, tag) VALUES ($1, $2)")
                .bind(id)
                .bind(tag)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(rec.0 as u32)
    }

    fn config(&self) -> &Config {
        &self.config
    }
}

