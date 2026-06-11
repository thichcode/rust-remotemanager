use dioxus::prelude::*;

#[component]
pub fn ConnectionForm() -> Element {
    rsx! {
        div { class: "connection-form",
            h3 { "Add Connection" }
            form {
                input { placeholder: "Host" }
                input { placeholder: "Port" }
                input { placeholder: "Username" }
                button { "Save" }
            }
        }
    }
}
