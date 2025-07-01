use dioxus::prelude::*;
use web_sys::{wasm_bindgen::JsCast, window, Text};

#[component]
pub fn Profile() -> Element {
    let save_token = |token| {
        let cookie_value = format!("token={}; path=/;", token);
        let _ = window().unwrap().document().unwrap().dyn_into::<web_sys::HtmlDocument>().unwrap().set_cookie(&cookie_value);
    };

    let get_token : fn() ->String = || {
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
             .and_then(|cookie| cookie.trim_start().split_once('=').map(|(_, val)| val.to_string()))
        })
        .unwrap_or_else(|| "".to_string())
    };

    let mut token = use_signal(|| get_token());

    let on_submit = move |_| {
        let  new_token = token();
        save_token(new_token)
    };

    rsx! {
        "Current token: " input { value: "{token}", oninput: move |e| token.set(e.value()), placeholder: "Enter your token here" }
        button { onclick: on_submit,
        "submit"}
    }
}
