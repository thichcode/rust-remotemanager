use dioxus::prelude::*;
use dioxus_router::prelude::*;
use crate::ui::{Route, state::AppState};

#[component]
pub fn Sidebar() -> Element {
    let state = use_context::<AppState>();
    let collapsed = state.sidebar_collapsed.lock();

    rsx! {
        aside { class: if *collapsed { "w-16 bg-gray-900" } else { "w-64 bg-gray-900" },
            div { class: "p-4",
                if !*collapsed {
                    h1 { class: "text-xl font-bold text-white", "Hermes" }
                }
            }
            nav { class: "mt-4",
                Link { to: Route::Dashboard {},
                    div { class: "px-4 py-2 text-gray-300 hover:bg-gray-800", "Dashboard" }
                }
                Link { to: Route::Connections {},
                    div { class: "px-4 py-2 text-gray-300 hover:bg-gray-800", "Connections" }
                }
                Link { to: Route::Settings {},
                    div { class: "px-4 py-2 text-gray-300 hover:bg-gray-800", "Settings" }
                }
            }
        }
    }
}
