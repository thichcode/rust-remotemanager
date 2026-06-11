use dioxus::prelude::*;
use crate::ui::state::AppState;

#[component]
pub fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let vault = state.vault.lock();
    let vault_status = if vault.is_unlocked() { "Unlocked" } else { "Locked" };

    rsx! {
        footer { class: "h-8 bg-gray-900 border-t border-gray-800 flex items-center px-4 text-xs text-gray-400",
            span { "Hermes Remote Manager" }
            span { class: "ml-auto", "Vault: {vault_status}" }
        }
    }
}
