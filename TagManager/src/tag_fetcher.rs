use std::io::Cursor;

use image::{DynamicImage, ImageOutputFormat};
use reqwest::multipart;

pub async fn fetch_tags(image: &DynamicImage) -> Result<Tags, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();

    image.write_to(&mut Cursor::new(&mut buffer), ImageOutputFormat::Png)?;

    let part = multipart::Part::bytes(buffer)
        .file_name("image.png")
        .mime_str("image/png")?;

    let form = multipart::Form::new().part("file", part);

    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8000/tag/")
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    return Ok(serde_json::from_str(&body)?);
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

impl Rating{
    pub fn to_dbformat(&self) ->&'static str{
        match self{
            &Rating::General => "general",
            &Rating::Sensitive => "sensitive",
            &Rating::Questionable => "questionable",
            &Rating::Explicit => "explicit",
        }
    }
}
