use dioxus::prelude::*;

#[component]
pub fn ConnectionsPage() -> Element {
    rsx! {
        div { class: "connections-page",
            h1 { "Connections" }
            p { "Manage your remote connections" }
        }
    }
}
