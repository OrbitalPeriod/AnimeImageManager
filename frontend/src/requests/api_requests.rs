use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct Paginated {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl Default for Paginated{
    fn default() -> Self {
        Self{
            page: None,
            per_page: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ImageRequest {
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FindImageRequest {
    pub characters: Option<String>,
    pub tags: Option<String>,
    pub rating: Option<Rating>,
    pub token: Option<String>,
    #[serde(flatten)]
    pub pages: Paginated,
}

impl Default for FindImageRequest{
    fn default() -> Self {
        Self{
            characters: None,
            tags: None,
            rating: None,
            token: None,
            pages: Paginated::default()
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct FindTagQuery {
    pub tag: Option<String>,
    #[serde(flatten)]
    pub pages: Paginated,
}

#[derive(Debug, Serialize, Clone)]
pub struct FindCharacterQuery{
    pub character: Option<String>,
    #[serde(flatten)]
    pub pages: Paginated,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Rating {
    General,
    Sensitive,
    Questionable,
    Explicit,
}
