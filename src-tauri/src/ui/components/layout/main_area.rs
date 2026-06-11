use dioxus::prelude::*;

#[component]
pub fn MainArea() -> Element {
    rsx! {
        div { class: "main-area",
            p { "Main content area" }
        }
    }
}
