use dioxus::prelude::*;

#[component]
pub fn SftpBrowser() -> Element {
    rsx! {
        div { class: "sftp-browser",
            h3 { "SFTP Browser" }
            p { "File transfer browser" }
        }
    }
}
