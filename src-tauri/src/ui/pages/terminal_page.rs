use dioxus::prelude::*;

#[component]
pub fn TerminalPage() -> Element {
    rsx! {
        div { class: "terminal-page",
            h1 { "Terminal" }
            p { "SSH terminal sessions" }
        }
    }
}
