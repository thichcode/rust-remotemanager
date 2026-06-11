use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    rsx! {
        div { class: "settings-page",
            h1 { "Settings" }
            p { "Application settings" }
        }
    }
}
