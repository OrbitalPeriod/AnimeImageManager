use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ApiResponse<T, E>
{
    pub status: u16,
    pub data: Result<T, E>,
}

#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    #[serde(flatten)]
    pub pages: Paginated,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Paginated {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u32,
    pub total_pages: u32,
    pub next: String,
    pub prev: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageData {
    pub id: i32,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct TagData {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Deserialize)]
pub struct CharacterData {
    pub name: String,
    pub count: u32,
}
