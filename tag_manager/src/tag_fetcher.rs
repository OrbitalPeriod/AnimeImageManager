use std::{error::Error, fmt, io::Cursor, sync::OnceLock};

use image::{DynamicImage, ImageOutputFormat};
use reqwest::multipart;

pub static TAGSERVICE_URL: OnceLock<String> = OnceLock::new();

pub async fn fetch_tags(image: &DynamicImage) -> Result<Tags, ImageFetcherError> {
    let mut buffer = Vec::new();

    image
        .write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png)
        .map_err(|e| ImageFetcherError(Box::new(e)))?;

    let part = multipart::Part::bytes(buffer)
        .file_name("image.png")
        .mime_str("image/png")
        .map_err(|e| ImageFetcherError(Box::new(e)))?;

    let form = multipart::Form::new().part("file", part);

    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8000/tag/")
        .multipart(form)
        .send()
        .await
        .map_err(|e| ImageFetcherError(Box::new(e)))?;

    let body = response
        .text()
        .await
        .map_err(|e| ImageFetcherError(Box::new(e)))?;

    serde_json::from_str(&body).map_err(|e| ImageFetcherError(Box::new(e)))
}

#[derive(Debug)]
pub struct ImageFetcherError(Box<dyn Error + Sync + Send>);

impl fmt::Display for ImageFetcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ImageFetcherError: {}", self.0)
    }
}

impl Error for ImageFetcherError{

}

impl ImageFetcherError {
    pub fn new<E>(error: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        ImageFetcherError(Box::new(error))
    }
}


#[derive(serde::Deserialize)]
pub struct Tags {
    pub rating: Rating,
    pub character_tags: Option<Vec<String>>,
    pub general_tags: Option<Vec<String>>,
}

#[derive(serde::Deserialize, sqlx::Type, Clone)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "rating", rename_all = "lowercase")]
pub enum Rating {
    General,
    Sensitive,
    Questionable,
    Explicit,
}
