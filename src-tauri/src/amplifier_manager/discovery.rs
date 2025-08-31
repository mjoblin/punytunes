//! Performs amplifier-specific UPnP discovery.
//!
//! Currently only attempts to discover Hegel amplifiers.

use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use futures::prelude::*;
use log::warn;
use log::Level::{Error, Info};
use pin_utils;
use rupnp::ssdp::{SearchTarget, URN};
use serde;
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::amplifier_manager::{AmplifierManagerAction, AmplifierManagerChannelMsg};
use crate::streammagic_manager::StreamMagicManagerChannelMsg;

const MEDIA_RENDERER: URN = URN::device("schemas-upnp-org", "MediaRenderer", 1);

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/AmplifierDevice.ts")]
pub struct AmplifierDevice {
    pub friendly_name: String,
    pub manufacturer: String,
    pub model: String,
    pub model_number: Option<String>,
    pub serial_number: Option<String>,
    pub url: String,
    pub udn: String,
}

impl fmt::Display for AmplifierDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}' ({}) [{}]", self.friendly_name, self.model, self.udn)
    }
}

pub async fn discover_amplifiers(
    amplifier_manager_channel: &mpsc::Sender<AmplifierManagerChannelMsg>,
    streammagic_manager_channel: &mpsc::Sender<StreamMagicManagerChannelMsg>,
) -> Result<(), rupnp::Error> {
    let amp_mgr = amplifier_manager_channel.clone();
    let streammagic_mgr = streammagic_manager_channel.clone();
    let search_target = SearchTarget::URN(MEDIA_RENDERER);
    let mut seen_udns: HashSet<String> = HashSet::new();
    let mut found_count = 0;

    send_app_log!(
        streammagic_mgr,
        Info,
        "Performing amplifier UPnP discovery (MediaRenderer only)"
    );

    match rupnp::discover(&search_target, Duration::from_secs(3)).await {
        Ok(discovered_devices) => {
            pin_utils::pin_mut!(discovered_devices);

            while let Some(device) = discovered_devices.try_next().await? {
                let device_udn = device.udn().to_string();

                if seen_udns.contains(&device_udn) {
                    continue;
                }

                seen_udns.insert(device_udn);

                if device.manufacturer() == "Hegel" {
                    found_count += 1;

                    let amplifier_device = AmplifierDevice {
                        friendly_name: device.friendly_name().to_string(),
                        manufacturer: device.manufacturer().to_string(),
                        model: device.model_name().to_string(),
                        model_number: device.model_number().map(|s| s.to_owned()),
                        serial_number: device.serial_number().map(|s| s.to_owned()),
                        url: device.url().to_string(),
                        udn: device.udn().to_string(),
                    };

                    send_app_log!(
                        streammagic_mgr,
                        Info,
                        "Amplifier device discovered: {} @ {}",
                        &amplifier_device,
                        &amplifier_device.url
                    );

                    match amplifier_manager_channel
                        .send(AmplifierManagerChannelMsg::AmplifierManagerActionMsg(
                            AmplifierManagerAction::ProcessDiscoveredDevice(amplifier_device),
                        ))
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            warn!(
                                "Could not send discovered device to AmplifierManager for processing: {:?}",
                                e
                            );
                        }
                    }
                } else {
                    send_app_log!(
                        streammagic_mgr,
                        Info,
                        "Amplifier UPnP discovery is ignoring MediaRenderer device '{}' ({}) from {}",
                        device.friendly_name(),
                        device.model_name(),
                        device.manufacturer(),
                    );
                }
            }
        }
        Err(e) => {
            send_app_log!(streammagic_mgr, Error, "Amplifier UPnP discovery error: {:?}", e);
        }
    }

    send_app_log!(
        streammagic_mgr,
        Info,
        "Amplifier UPnP discovery found {} amplifier{}",
        &found_count,
        if found_count > 1 { "s" } else { "" },
    );

    send_amplifier_manager_action!(&amp_mgr, AmplifierManagerAction::SetIsDiscovering(false));

    Ok(())
}
