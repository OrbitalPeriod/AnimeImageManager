use crate::components::Image;
use crate::{
    requests::api_requests::{FindImageRequest, Paginated, Rating as otherRating},
    responses::api_response::{ApiResponse, ImageData, PaginatedResponse},
};
use dioxus::prelude::*;
use itertools::Itertools;
use reqwest::Client;
use serde::Deserialize;
use web_sys::wasm_bindgen::JsCast;
use web_sys::window;

#[derive(Default)]
struct QueryParams {
    characters: String,
    tags: String,
    rating: Rating,
}

#[derive(Deserialize, Default, Clone, Copy, PartialEq, Eq)]
enum Rating {
    General,
    Sensitive,
    Questionable,
    Explicit,
    #[default]
    All,
}

impl Rating {
    fn into_apirating(self) -> Option<otherRating> {
        match self {
            Self::General => Some(otherRating::General),
            Self::Sensitive => Some(otherRating::Sensitive),
            Self::Questionable => Some(otherRating::Questionable),
            Self::Explicit => Some(otherRating::Explicit),
            Self::All => None,
        }
    }
    fn from_str(str: &str) -> Option<Self> {
        match str {
            "General" => Some(Self::General),
            "Sensitive" => Some(Self::Sensitive),
            "Questionable" => Some(Self::Questionable),
            "Explicit" => Some(Self::Explicit),
            "All" => Some(Self::All),
            _ => None,
        }
    }
}

/// The Blog page component that will be rendered when the current route is `[Route::Blog]`
///
/// The component takes a `id` prop of type `i32` from the route enum. Whenever the id changes, the component function will be
/// re-run and the rendered HTML will be updated.
#[component]
pub fn Search() -> Element {
    let mut char_param = use_signal(|| "".to_string());
    let mut tag_param = use_signal(|| "".to_string());
    let mut rating_param = use_signal(Rating::default);

    let mut search_result = use_signal(|| Vec::<ImageData>::new());

    let mut on_search = move |_| {
        let get_token: fn() -> Option<String> = || {
            window()
                .unwrap()
                .document()
                .unwrap()
                .dyn_into::<web_sys::HtmlDocument>()
                .unwrap()
                .cookie()
                .ok()
                .and_then(|x| {
                    x.split(';')
                        .find(|cookie| cookie.trim_start().starts_with("token="))
                        .and_then(|cookie| {
                            cookie
                                .trim_start()
                                .split_once('=')
                                .map(|(_, val)| val.to_string())
                        })
                })
        };

        let chars = char_param();
        let tags = tag_param();
        let rating = rating_param();

        let query = FindImageRequest {
            characters: Some(chars.split_whitespace().intersperse(",").collect()),
            tags: Some(tags.split_whitespace().intersperse(",").collect()),
            rating: rating.into_apirating(),
            pages: Paginated {
                per_page: Some(20),
                page: Some(0),
            },
            token: get_token(),
            ..Default::default()
        };
        let query_string = serde_urlencoded::to_string(query).unwrap();

        spawn(async move {
            let query_string = format!("http://127.0.0.1:8080/search?{}", query_string);
            let client = Client::new();
            let response = client.get(&query_string).send().await;
            let data = response.unwrap();

            tracing::info!("Pinging api with query: {}", query_string);

            let data = data
                .json::<ApiResponse<PaginatedResponse<ImageData>, String>>()
                .await
                .unwrap();
            let results = match data.data {
                Ok(data) => {
                    tracing::info!("Got items: {:?}", data);
                    data.items
                }
                Err(e) => {
                    tracing::info!("Soemthing went wrong trying to fetch items: {}", e);
                    Vec::new()
                }
            };
            search_result.set(results);
        });
    };

    rsx! {
     input { value: "{char_param}", oninput: move |event| char_param.set(event.value())}
     input { value: "{tag_param}", oninput: move |event| tag_param.set(event.value())}
     select {
         oninput: move |event| {rating_param.set(Rating::from_str(&event.value()).unwrap());},
         option {"General"},
         option {"Sensitive"},
         option {"Questionable"},
         option {"Explicit"},
         option{"All"},
    },
    input { r#type: "submit", onclick: on_search, }
         div {for item in search_result.read().iter() {
             Image {url: item.url.clone()}
         }}
     }
}
