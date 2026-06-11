use dioxus::prelude::*;

#[component]
pub fn TerminalTab() -> Element {
    rsx! {
        div { class: "terminal-tab",
            span { "Terminal Tab" }
        }
    }
}
