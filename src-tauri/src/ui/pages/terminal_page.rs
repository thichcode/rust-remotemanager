use dioxus::prelude::*;
use crate::ui::components::terminal::terminal_session::TerminalSession;

#[component]
pub fn TerminalPage(session_id: String) -> Element {
    rsx! {
        div { class: "h-full",
            TerminalSession { session_id: session_id.clone() }
        }
    }
}