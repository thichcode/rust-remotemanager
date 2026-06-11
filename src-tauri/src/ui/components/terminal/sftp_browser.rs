use dioxus::prelude::*;

#[component]
pub fn SftpBrowser(session_id: String) -> Element {
    rsx! {
        div { class: "w-80 bg-gray-900 border-l border-gray-800 flex flex-col",
            div { class: "p-2 border-b border-gray-800 text-sm",
                span { class: "text-gray-400", "Remote: /home" }
            }
            div { class: "flex-1 overflow-auto p-2",
                div { class: "text-gray-400 text-sm", "SFTP browser coming soon..." }
            }
        }
    }
}
