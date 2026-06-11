use dioxus::prelude::*;

#[component]
pub fn TerminalSession() -> Element {
    rsx! {
        div { class: "terminal-session",
            pre { "Terminal output will appear here" }
        }
    }
}
