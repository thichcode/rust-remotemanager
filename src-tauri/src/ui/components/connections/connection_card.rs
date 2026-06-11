use dioxus::prelude::*;

#[component]
pub fn ConnectionCard(name: String, host: String, port: i32, conn_type: String) -> Element {
    let type_icon = match conn_type.as_str() {
        "ssh" => "🖥️",
        "rdp" => "🖥️",
        "serial" => "🔌",
        _ => "📡",
    };

    rsx! {
        div { class: "bg-gray-800 rounded-lg p-4 hover:bg-gray-700 cursor-pointer",
            div { class: "flex items-center justify-between mb-2",
                span { class: "text-lg", "{type_icon}" }
            }
            div { class: "font-medium", "{name}" }
            div { class: "text-sm text-gray-400", "{host}:{port}" }
        }
    }
}
