use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use futures::prelude::*;
use log::{warn};
use log::Level::{Info, Error};
use pin_utils;
use rupnp::ssdp::{SearchTarget, URN};
use serde;
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::streammagic_manager::{StreamMagicManagerAction, StreamMagicManagerChannelMsg};

const MEDIA_RENDERER: URN = URN::device("schemas-upnp-org", "MediaRenderer", 1);

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/StreamMagicDevice.ts")]
pub struct StreamMagicDevice {
    pub friendly_name: String,
    pub model: String,
    pub model_number: Option<String>,
    pub serial_number: Option<String>,
    pub url: String,
    pub udn: String,
    pub is_activating: bool,
    pub is_active: bool,
}

impl fmt::Display for StreamMagicDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}' ({}) [{}]", self.friendly_name, self.model, self.udn)
    }
}

pub async fn discover_streamers(
    manager_channel: &mpsc::Sender<StreamMagicManagerChannelMsg>,
    activate_discovered_device: bool,
) -> Result<(), rupnp::Error> {
    let mgr = manager_channel.clone();
    let search_target = SearchTarget::URN(MEDIA_RENDERER);
    let mut seen_udns: HashSet<String> = HashSet::new();
    let mut found_count = 0;

    send_app_log!(mgr, Info, "Performing streamer UPnP discovery (MediaRenderer only)");

    match rupnp::discover(&search_target, Duration::from_secs(3)).await {
        Ok(discovered_devices) => {
            pin_utils::pin_mut!(discovered_devices);

            while let Some(device) = discovered_devices.try_next().await? {
                let device_udn = device.udn().to_string();

                if seen_udns.contains(&device_udn) {
                    continue;
                }

                seen_udns.insert(device_udn);

                if device.manufacturer() == "Cambridge Audio" {
                    found_count += 1;

                    let streammagic_device = StreamMagicDevice {
                        friendly_name: device.friendly_name().to_string(),
                        model: device.model_name().to_string(),
                        model_number: device.model_number().map(|s| s.to_owned()),
                        serial_number: device.serial_number().map(|s| s.to_owned()),
                        url: device.url().to_string(),
                        udn: device.udn().to_string(),
                        is_activating: false,
                        is_active: false,
                    };

                    send_app_log!(mgr, Info,
                        "StreamMagic device discovered: {} @ {}",
                        &streammagic_device, &streammagic_device.url
                    );

                    match manager_channel
                        .send(StreamMagicManagerChannelMsg::StreamMagicManagerActionMsg(
                            StreamMagicManagerAction::ProcessDiscoveredDevice(streammagic_device),
                        ))
                        .await
                    {
                        Ok(_) => {
                            if activate_discovered_device {
                                match manager_channel
                                    .send(StreamMagicManagerChannelMsg::StreamMagicManagerActionMsg(
                                        StreamMagicManagerAction::ActivateUdn(device.udn().to_string()),
                                    ))
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(e) => {
                                        warn!("Could not send UDN activation request to StreamMagicManager: {:?}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Could not send discovered device to StreamMagicManager for processing: {:?}",
                                e
                            );
                        }
                    }
                } else {
                    send_app_log!(mgr, Info,
                        "Streamer UPnP discovery is ignoring MediaRenderer device '{}' ({}) from {}",
                        device.friendly_name(),
                        device.model_name(),
                        device.manufacturer(),
                    );
                }
            }
        }
        Err(e) => {
            send_app_log!(mgr, Error, "Streamer UPnP discovery error: {:?}", e);
        }
    }

    send_app_log!(
        mgr,
        Info,
        "Streamer UPnP discovery found {} streamer{}",
        &found_count,
        if found_count > 1 { "s" } else { "" },
    );

    send_manager_action!(&mgr, StreamMagicManagerAction::SetIsDiscovering(false));

    Ok(())
}
