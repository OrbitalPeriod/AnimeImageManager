use std::fmt::format;

use actix_web::{HttpResponse, Responder, body::BoxBody, http::StatusCode};
use serde::Serialize;
use serde_json::json;

pub struct ApiResponse<T: Serialize, E: Serialize> {
    status: StatusCode,
    data: ApiData<T, E>,
}

pub enum ApiData<T: Serialize, E: Serialize> {
    Json(Result<T, E>),
    Binary(Vec<u8>, String), // data, content_type
}

impl<T: Serialize, E: Serialize> ApiResponse<T, E> {
    pub fn new(status: StatusCode, data: ApiData<T, E>) -> Self {
        Self { status, data }
    }
    pub fn new_json(status: StatusCode, data: Result<T, E>) -> Self {
        Self::new(status, ApiData::Json(data))
    }
    pub fn new_success(data: T) -> Self {
        Self::new_json(StatusCode::from_u16(200).unwrap(), Ok(data))
    }
    pub fn new_internal_server_error(error: E) -> Self {
        Self::new_json(StatusCode::from_u16(500).unwrap(), Err(error))
    }
    pub fn new_bad_request(error: E) -> Self {
        Self::new_json(StatusCode::BAD_REQUEST, Err(error))
    }
    pub fn new_not_allowed(error: E) -> Self {
        Self::new_json(StatusCode::METHOD_NOT_ALLOWED, Err(error))
    }
    pub fn new_binary(status: StatusCode, content: Vec<u8>, content_type: &str) -> Self {
        Self::new(status, ApiData::Binary(content, content_type.to_string()))
    }
}

impl<T: Serialize, E: Serialize> Responder for ApiResponse<T, E> {
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match self.data {
            ApiData::Json(ref result) => {
                let body = match result {
                    Ok(_) => json!({"status": self.status.as_u16() , "data": result}),
                    Err(_) => json!({"status": self.status.as_u16(), "data": result}),
                };
                HttpResponse::build(self.status)
                    .content_type("application/json")
                    .body(body.to_string())
            }
            ApiData::Binary(content, content_type) => HttpResponse::build(self.status)
                .content_type(content_type)
                .body(content),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    #[serde(flatten)]
    pub pages: Paginated,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, prefix: &str, page: u32, per_page: u32, total_items: u32) -> Self {
        Self {
            items,
            pages: Paginated::new(prefix, page, per_page, total_items),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Paginated {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u32,
    pub total_pages: u32,
    pub next: String,
    pub prev: String,
}

impl Paginated {
    fn new(prefix: &str, page: u32, per_page: u32, total_items: u32) -> Self {
        let total_pages = total_items.div_ceil(per_page);
        Self {
            page,
            per_page,
            total_pages,
            total_items,
            next: format!(
                "{}&page={}&per_page={}",
                prefix,
                {
                    if page >= total_pages {
                        total_pages
                    } else {
                        page + 1
                    }
                },
                per_page
            ),
            prev: format!("{}&page={}&per_page={}", prefix, page.saturating_sub(1), per_page),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Imagedata {
    id: i32,
    url: String,
}

impl Imagedata {
    pub fn new(id: i32, url: String) -> Self {
        Self { id, url }
    }
}

#[derive(Debug, Serialize)]
pub struct TagData {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Serialize)]
pub struct CharacterData{
    pub name: String,
    pub count: u32
}
