use dioxus::prelude::*;

#[component]
pub fn FolderTree() -> Element {
    rsx! {
        div { class: "space-y-1",
            div { class: "px-2 py-1 text-gray-400", "No folders" }
        }
    }
}
