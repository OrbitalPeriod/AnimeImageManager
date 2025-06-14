use std::path::PathBuf;

pub trait Database{
    async fn get_image_location(&self, id : u32) -> Result<PathBuf, SqlDatabaseError>;
}

#[derive(Clone, Debug)]
pub struct SqlDatabase{
    pool : sqlx::postgres::PgPool,
    path : PathBuf,
}

impl SqlDatabase{
    pub async fn new(connection_string : &str, path: PathBuf) -> Result<Self, sqlx::error::Error>{
        let pool = sqlx::postgres::PgPool::connect(connection_string).await?;

        Ok(Self{
            pool,
            path: path.clone()
        })
    }
}

impl Database for SqlDatabase{
     async fn get_image_location(&self, id : u32) -> Result<PathBuf, SqlDatabaseError>{
        let test : i32 = sqlx::query_scalar!("SELECT id FROM image WHERE id = $1 LIMIT 1", id as i32)
            .fetch_optional(&self.pool)
            .await.map_err(|x| SqlDatabaseError::SqlxError(x))?.ok_or(SqlDatabaseError::FileNotFound)?;

        let path = self.path.join(format!("{}..png", test));

        Ok(path)
    }
}

#[derive(Debug)]
pub enum SqlDatabaseError{
    FileNotFound,
    SqlxError(sqlx::error::Error)
}
