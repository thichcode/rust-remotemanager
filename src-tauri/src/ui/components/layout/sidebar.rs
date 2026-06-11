use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        div { class: "sidebar",
            h2 { "Navigation" }
            ul {
                li { "Dashboard" }
                li { "Connections" }
                li { "Terminal" }
                li { "Settings" }
            }
        }
    }
}
