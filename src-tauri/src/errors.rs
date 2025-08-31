#[derive(Debug, thiserror::Error)]
pub enum PunyTunesError {
    #[error("The device with UDN '{0}' is not in the list of discovered devices")]
    UnknownDevice(String),

    #[error("{0}")]
    Store(String),

    #[error(transparent)]
    UPnP(#[from] rupnp::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tauri(#[from] tauri::Error),

    #[error("{0}")]
    WebSocket(String),

    #[error("WebSocket connection closed by server")]
    WebSocketConnectionClosed,

    #[error("WebSocket timeout")]
    WebSocketTimeout,

    #[error("WebSocket client has lost its connection to the server")]
    WebSocketClientLostConnection,

    #[error("{0}")]
    Amplifier(String),
}

impl serde::Serialize for PunyTunesError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
