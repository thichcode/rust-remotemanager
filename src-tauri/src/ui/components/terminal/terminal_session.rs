use dioxus::prelude::*;
use super::terminal_emulator::TerminalEmulator;

#[component]
pub fn TerminalSession(session_id: String) -> Element {
    let emulator = use_signal(|| TerminalEmulator::new(80, 24));

    rsx! {
        div { class: "h-full bg-black p-2 font-mono text-sm text-green-500 overflow-auto",
            pre { class: "whitespace-pre-wrap", "{emulator.read().render()}" }
        }
    }
}
