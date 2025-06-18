use actix_web::{body::BoxBody, http::StatusCode, HttpResponse, Responder};
use serde::Serialize;
use serde_json::json;

pub struct ApiResponse<T : Serialize, E : Serialize>{
    status : StatusCode,
    data : Result<T, E>
}

impl<T: Serialize, E:Serialize> ApiResponse<T, E>{
    pub fn new(status: StatusCode, data: Result<T, E>) -> Self{
        Self{
            status,
            data,
        }
    }
    pub fn new_internal_server_error(error: E) -> Self{
        Self::new(StatusCode::from_u16(500).unwrap(), Err(error))
    }
    pub fn new_success(data : T) -> Self{
        Self::new(StatusCode::from_u16(200).unwrap(), Ok(data))
    }
}

impl<T: Serialize, E : Serialize> Responder for ApiResponse<T, E>{
    type Body = BoxBody;

    fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let body = match self.data{
            Ok(value) => json!({"status": "success", "data": value}),
            Err(error_message) => json!({"status": "error", "error": error_message}),
        };

        HttpResponse::build(self.status).content_type("application/json").body(body.to_string())
    }
}

pub type FindImageResponse = Vec<Imagedata>;

#[derive(Debug, Serialize)]
pub struct Imagedata{
    id: i32,
    url: String,
}

impl Imagedata{
    pub fn new(id: i32, url: String) -> Self{
        Self{
            id,
            url
        }
    }
}
