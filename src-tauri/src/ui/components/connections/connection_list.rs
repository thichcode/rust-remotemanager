use dioxus::prelude::*;

#[component]
pub fn ConnectionList() -> Element {
    rsx! {
        div { class: "connection-list",
            h3 { "Connections" }
            p { "List of saved connections" }
        }
    }
}
