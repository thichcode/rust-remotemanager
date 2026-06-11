use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div { class: "p-6",
            h1 { class: "text-2xl font-bold mb-6", "Dashboard" }

            div { class: "grid grid-cols-3 gap-4 mb-6",
                div { class: "bg-gray-800 rounded-lg p-4",
                    div { class: "text-gray-400 text-sm", "Total Connections" }
                    div { class: "text-3xl font-bold", "0" }
                }
                div { class: "bg-gray-800 rounded-lg p-4",
                    div { class: "text-gray-400 text-sm", "Active Sessions" }
                    div { class: "text-3xl font-bold text-green-500", "0" }
                }
                div { class: "bg-gray-800 rounded-lg p-4",
                    div { class: "text-gray-400 text-sm", "Favorites" }
                    div { class: "text-3xl font-bold text-yellow-500", "0" }
                }
            }

            h2 { class: "text-lg font-semibold mb-4", "Recent Connections" }
            div { class: "text-gray-400", "No connections yet. Go to Connections to add one." }
        }
    }
}