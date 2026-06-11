use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div { class: "dashboard",
            h1 { "Dashboard" }
            p { "Welcome to Hermes Remote Manager" }
        }
    }
}
