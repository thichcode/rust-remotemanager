use dioxus::prelude::*;
use dioxus_router::prelude::*;

pub mod state;
pub mod theme;
pub mod pages;
pub mod components;
pub mod services;

use pages::dashboard::Dashboard;
use pages::connections::Connections;
use pages::terminal_page::TerminalPage;
use pages::settings::Settings;
use components::layout::sidebar::Sidebar;
use components::layout::status_bar::StatusBar;
use components::terminal::terminal_tab::TerminalTabs;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Dashboard {},
    #[route("/connections")]
    Connections {},
    #[route("/settings")]
    Settings {},
    #[route("/terminal/:session_id")]
    TerminalPage { session_id: String },
}

#[component]
pub fn App() -> Element {
    rsx! {
        div { class: "flex h-screen bg-gray-950 text-white",
            Sidebar {}
            div { class: "flex-1 flex flex-col",
                TerminalTabs {}
                main { class: "flex-1 overflow-auto",
                    Router::<Route> {}
                }
                StatusBar {}
            }
        }
    }
}