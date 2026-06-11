use tokio::sync::broadcast;

#[derive(Clone)]
pub struct EventBridge {
    pub terminal_output: broadcast::Sender<(String, String)>,
    pub terminal_connected: broadcast::Sender<(String, u16, u16)>,
    pub terminal_error: broadcast::Sender<(String, String)>,
    pub terminal_exit: broadcast::Sender<String>,
}

impl EventBridge {
    pub fn new() -> Self {
        let (terminal_output, _) = broadcast::channel(256);
        let (terminal_connected, _) = broadcast::channel(32);
        let (terminal_error, _) = broadcast::channel(32);
        let (terminal_exit, _) = broadcast::channel(32);

        Self {
            terminal_output,
            terminal_connected,
            terminal_error,
            terminal_exit,
        }
    }
}
