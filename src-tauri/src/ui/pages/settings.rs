use dioxus::prelude::*;

#[component]
pub fn Settings() -> Element {
    let mut active_tab = use_signal(|| "general".to_string());

    rsx! {
        div { class: "p-6 max-w-4xl mx-auto",
            h1 { class: "text-2xl font-bold mb-6", "Settings" }

            div { class: "flex gap-4 border-b border-gray-800 mb-6",
                for tab in ["general", "appearance", "security", "terminal", "about"] {
                    button {
                        key: "{tab}",
                        class: if active_tab() == tab {
                            "pb-2 border-b-2 border-blue-500 text-white"
                        } else {
                            "pb-2 text-gray-400 hover:text-white"
                        },
                        onclick: move |_| *active_tab.write() = tab.to_string(),
                        "{tab}"
                    }
                }
            }

            match active_tab.read().as_str() {
                "general" => rsx! { div { "General settings coming soon..." } },
                "appearance" => rsx! { div { "Appearance settings coming soon..." } },
                "security" => rsx! { div { "Security settings coming soon..." } },
                "terminal" => rsx! { div { "Terminal settings coming soon..." } },
                "about" => rsx! {
                    div {
                        h3 { class: "font-semibold", "Hermes Remote Manager" }
                        p { class: "text-gray-400", "Version 0.1.0" }
                    }
                },
                _ => rsx! { div { "Unknown tab" } }
            }
        }
    }
}