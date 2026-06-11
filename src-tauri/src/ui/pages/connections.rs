use dioxus::prelude::*;
use crate::ui::components::connections::folder_tree::FolderTree;

#[component]
pub fn Connections() -> Element {
    let mut show_form = use_signal(|| false);

    rsx! {
        div { class: "flex h-full",
            div { class: "w-64 bg-gray-900 border-r border-gray-800 p-4",
                h3 { class: "text-sm font-semibold text-gray-400 mb-2", "Folders" }
                FolderTree {}
            }

            div { class: "flex-1 p-4",
                div { class: "flex items-center justify-between mb-4",
                    input {
                        class: "bg-gray-800 rounded px-3 py-2 w-64",
                        placeholder: "Search connections...",
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 rounded hover:bg-blue-500",
                        onclick: move |_| *show_form.write() = true,
                        "+ New Connection"
                    }
                }

                div { class: "text-gray-400", "No connections yet." }
            }

            if show_form() {
                // ConnectionForm placeholder
            }
        }
    }
}