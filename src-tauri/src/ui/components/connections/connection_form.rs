use dioxus::prelude::*;

#[component]
pub fn ConnectionForm() -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            div { class: "bg-gray-800 rounded-lg p-6 w-full max-w-md",
                h2 { class: "text-xl font-bold mb-4", "New Connection" }
                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm text-gray-400 mb-1", "Name" }
                        input { class: "w-full bg-gray-700 rounded px-3 py-2" }
                    }
                    div {
                        label { class: "block text-sm text-gray-400 mb-1", "Host" }
                        input { class: "w-full bg-gray-700 rounded px-3 py-2" }
                    }
                }
            }
        }
    }
}
