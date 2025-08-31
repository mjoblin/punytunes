//! Items to aid and support Amplifier Handlers.
//!
//! An Amplifier Handler is responsible for communicating with an amplifier. It should implement
//! the `AmplifierHandler` trait. An `AmplifierHandler` will be provided with Rx and Tx channels
//! for receiving and sending messages from/to the `AmplifierManager`. The Handler's work should
//! all be done in the async `run()` function.
//!
//! Handlers should handle incoming messages containing `AmplifierHandlerAction` and
//! `AmplifierAction` enum variants. Handlers should also manage the `AmplifierState` for their
//! amplifier, and send `AmplifierState` and `AmplifierHandlerConnectionStatus` messages back
//! to the `AmplifierManager`.

use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};
use ts_rs::TS;

use crate::amplifier_manager::discovery::AmplifierDevice;
use crate::amplifier_manager::AmplifierAction;
use crate::errors::PunyTunesError;
use crate::streammagic_manager::StreamMagicManagerChannelMsg;

#[derive(Clone, Debug, serde::Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/AmplifierState.ts")]
pub struct AmplifierState {
    pub is_muted: Option<bool>,
    pub is_powered_on: Option<bool>,
    pub source: Option<u8>,
    pub volume: Option<u8>,
}

impl Default for AmplifierState {
    fn default() -> Self {
        AmplifierState {
            is_muted: None,
            is_powered_on: None,
            source: None,
            volume: None,
        }
    }
}

/// Messages sent to the `AmplifierHandler` from the `AmplifierManager`.
#[derive(Clone, Debug)]
pub enum AmplifierHandlerRxChannelMsg {
    AmplifierActionMsg(AmplifierAction),
    AmplifierHandlerActionMsg(AmplifierHandlerAction),
}

#[derive(Clone, Debug)]
pub enum AmplifierHandlerAction {
    ShutDown,
    TestConnection,
}

/// Messages sent from the `AmplifierHandler` to the `AmplifierManager`.
#[derive(Clone, Debug)]
pub enum AmplifierHandlerTxChannelMsg {
    AmplifierStateMsg(AmplifierState),
    AmplifierHandlerConnectionStatusMsg(AmplifierHandlerConnectionStatus),
}

#[derive(Clone, Debug)]
pub enum AmplifierHandlerConnectionStatus {
    Connected,
    Disconnected,
}

/// Trait to be implemented by an Amplifier Handler.
#[async_trait]
pub trait AmplifierHandler {
    fn new(
        device: AmplifierDevice,
        rx_channel: Receiver<AmplifierHandlerRxChannelMsg>,
        tx_channel: Sender<AmplifierHandlerTxChannelMsg>,
        streammagic_manager_channel: Sender<StreamMagicManagerChannelMsg>,
    ) -> Self;

    async fn run(&mut self) -> Result<(), PunyTunesError>;
}
