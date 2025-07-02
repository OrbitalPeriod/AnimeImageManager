use anyhow::Result;

use crate::{
    Config,
    tag_fetcher::{Rating, Tags},
};

pub trait Database {
    async fn create(config: &Config) -> Result<impl Database + Clone>;
    async fn save_image(&self, hash: &[u8; 8], tags: &Tags) -> Result<u32>;
    async fn check_hash(&self, hash: &[u8; 8]) -> Result<bool>;
    fn config(&self) -> &Config;
    async fn get_non_thumbnailed_images(&self) -> Result<Vec<u32>>;
    async fn write_thumbnail(&self, id: u32) -> Result<()>;
}

#[derive(Clone)]
pub struct SqlDatabase {
    pool: sqlx::postgres::PgPool,
    config: Config,
}

impl Database for SqlDatabase {
    async fn create(config: &Config) -> Result<impl Database + Clone> {
        Ok(Self {
            pool: sqlx::postgres::PgPool::connect(&config.connection_string).await?,
            config: config.clone(),
        })
    }

    async fn check_hash(&self, hash: &[u8; 8]) -> Result<bool> {
        let query: (bool,) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM image WHERE hash=$1)")
            .bind(hash)
            .fetch_one(&self.pool)
            .await?;

        Ok(query.0)
    }
    async fn save_image(&self, hash: &[u8; 8], tags: &Tags) -> Result<u32> {
        let rec: (i32,) =
            sqlx::query_as("INSERT INTO image (rating, hash) VALUES ($1, $2) RETURNING id")
                .bind(tags.rating.clone() as Rating)
                .bind(hash)
                .fetch_one(&self.pool)
                .await?;

        let id = rec.0;

        if let Some(character_tags) = &tags.character_tags {
            for tag in character_tags {
                let tag_id = self.get_character_tag_id(tag).await?;

                sqlx::query!(
                    "INSERT INTO character_images (image_id, character_id) VALUES ($1, $2)",
                    id,
                    tag_id
                )
                .execute(&self.pool)
                .await?;
            }
        }
        if let Some(general_tags) = &tags.general_tags {
            for tag in general_tags {
                let tag_id = self.get_general_tag_id(tag).await?;

                sqlx::query!(
                    "INSERT INTO tag_images (image_id, tag_id) VALUES ($1, $2)",
                    id,
                    tag_id
                )
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(rec.0 as u32)
    }

    fn config(&self) -> &Config {
        &self.config
    }

    async fn get_non_thumbnailed_images(&self) -> Result<Vec<u32>> {
        Ok(sqlx::query!("SELECT id from image where thumbnail=false")
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|x| x.id as u32)
            .collect())
    }

    async fn write_thumbnail(&self, id: u32) -> Result<()> {
        sqlx::query!("UPDATE image SET thumbnail=true WHERE id=$1;", id as i64).execute(&self.pool).await?;
        Ok(())
    }
}

impl SqlDatabase {
    async fn get_character_tag_id(&self, character_name: &str) -> Result<i32> {
        let record = sqlx::query!(
            r#"
            WITH ins AS (
                INSERT INTO "character" (character)
                VALUES ($1)
                ON CONFLICT (character) DO NOTHING
                RETURNING id
            )
            SELECT id FROM ins 
            UNION ALL 
            SELECT id FROM "character" WHERE character = $1
            LIMIT 1
            "#,
            character_name
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record.id.expect("what"))
    }
    async fn get_general_tag_id(&self, tag: &str) -> Result<i32> {
        let record = sqlx::query!(
            r#"
            WITH ins AS (
                INSERT INTO "tag" (tag)
                VALUES ($1)
                ON CONFLICT (tag) DO NOTHING
                RETURNING id
            )
            SELECT id FROM ins 
            UNION ALL 
            SELECT id FROM "tag" WHERE tag = $1
            LIMIT 1
            "#,
            tag
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(record.id.expect("what"))
    }
}
