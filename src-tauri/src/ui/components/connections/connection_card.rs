use dioxus::prelude::*;

#[component]
pub fn ConnectionCard() -> Element {
    rsx! {
        div { class: "connection-card",
            h4 { "Connection" }
            p { "Connection details" }
        }
    }
}
