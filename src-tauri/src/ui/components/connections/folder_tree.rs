use dioxus::prelude::*;

#[component]
pub fn FolderTree() -> Element {
    rsx! {
        div { class: "folder-tree",
            h3 { "Folders" }
            ul {
                li { "All Connections" }
                li { "Favorites" }
            }
        }
    }
}
