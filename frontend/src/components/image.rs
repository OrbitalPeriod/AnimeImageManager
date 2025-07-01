use dioxus::prelude::*;

const IMAGE_CSS: Asset = asset!("/assets/styling/image.css");

#[component]
pub fn Image(url: String) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: IMAGE_CSS }
        img{
            src: url,
        }
    }
}
