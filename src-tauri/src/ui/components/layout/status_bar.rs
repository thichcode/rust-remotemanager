use dioxus::prelude::*;

#[component]
pub fn StatusBar() -> Element {
    rsx! {
        div { class: "status-bar",
            span { "Hermes Remote Manager v0.1.0" }
        }
    }
}
