use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{async_runtime, Manager, Runtime};
use tauri_plugin_pinia::ManagerExt;
use tauri_specta::Event;
use tokio::{select, sync::mpsc, time};
use tpower::{
    ffi::smc::{SMCConnection, SMCPowerData, SMCReadSensor},
    provider::{get_mac_ioreg, NormalizedResource},
};

use crate::event::{PowerUpdatedEvent, PreferenceEvent, StatusBarItem, WindowLoadedEvent};

pub enum SenderMessage {
    ImmediateSend,
    ChangeInterval(Duration),
    ChangeStatusBarItem(StatusBarItem),
    StatusBarShowCharging(bool),
    WindowVisibilityChanged(bool),
}

pub fn status_bar_text(
    smc: &SMCPowerData,
    status_bar_item: &StatusBarItem,
    show_charging: bool,
) -> f32 {
    if smc.is_charging() && show_charging {
        return smc.delivery_rate;
    }
    match status_bar_item {
        StatusBarItem::System => smc.system_total,
        StatusBarItem::Screen => smc.brightness,
        StatusBarItem::Heatpipe => smc.heatpipe,
    }
}

impl PowerUpdatedEvent {
    pub fn new(value: f32) -> Self {
        Self(format!("{:.1} w", value))
    }

    pub fn new_with(
        smc: &SMCPowerData,
        status_bar_item: &StatusBarItem,
        show_charging: bool,
    ) -> Self {
        Self::new(status_bar_text(smc, status_bar_item, show_charging))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Event, Type)]
#[serde(rename_all = "camelCase")]
pub struct PowerTickEvent {
    pub data: NormalizedResource,
}

pub fn start_sender<R: Runtime>(
    app: &impl Manager<R>,
    mut rx: mpsc::Receiver<SenderMessage>,
) -> async_runtime::JoinHandle<()> {
    let app = app.app_handle().clone();
    let mut smc_conn = SMCConnection::new("AppleSMC").unwrap();

    let mut timer = time::interval(Duration::from_millis(
        app.pinia()
            .try_get::<u64>("preference", "updateInterval")
            .unwrap_or(3000),
    ));
    let mut status_bar_item = app
        .pinia()
        .try_get::<StatusBarItem>("preference", "statusBarItem")
        .unwrap_or(StatusBarItem::System);
    let mut show_charging = app
        .pinia()
        .try_get::<bool>("preference", "showCharging")
        .unwrap_or(true);

    async_runtime::spawn(async move {
        let mut last_system_total: f32 = 0.0;
        let mut last_battery_level: f32 = 0.0;
        let mut last_charging = false;
        let mut any_window_visible = false;

        loop {
            select! {
                _ = timer.tick() => {
                    let smc = smc_conn.read_sensor();

                    // Always emit tray update (throttled in tray_icon.rs)
                    PowerUpdatedEvent::new_with(&smc, &status_bar_item, show_charging)
                        .emit(&app)
                        .unwrap();

                    // Skip frontend events entirely when no windows are visible
                    if !any_window_visible {
                        continue;
                    }

                    // Only emit PowerTickEvent when data changes meaningfully
                    let charging_changed = smc.is_charging() != last_charging;
                    let power_delta = (smc.system_total - last_system_total).abs();
                    let battery_delta = (smc.current_capacity - last_battery_level).abs();

                    if charging_changed || power_delta > 0.3 || battery_delta > 1.0 {
                        PowerTickEvent {
                            data: (&get_mac_ioreg().unwrap(), &smc).into(),
                        }.emit(&app).unwrap();

                        last_system_total = smc.system_total;
                        last_battery_level = smc.current_capacity;
                        last_charging = smc.is_charging();
                    }
                }
                Some(msg) = rx.recv() => match msg {
                    SenderMessage::ImmediateSend => {
                        let smc = smc_conn.read_sensor();
                        PowerUpdatedEvent::new_with(&smc, &status_bar_item, show_charging)
                            .emit(&app)
                            .unwrap();
                        PowerTickEvent {
                            data:  (&get_mac_ioreg().unwrap(), &smc).into()
                        }.emit(&app).unwrap();
                        last_system_total = smc.system_total;
                        last_battery_level = smc.current_capacity;
                        last_charging = smc.is_charging();
                    },
                    SenderMessage::ChangeInterval(interval) => {
                        timer = time::interval(if interval < Duration::from_millis(500) {
                            log::warn!("interval is too small, set to 500ms");
                            Duration::from_millis(500)
                        } else {
                            interval
                        });
                    },
                    SenderMessage::ChangeStatusBarItem(item) => {
                        status_bar_item = item;
                        PowerUpdatedEvent::new_with(&smc_conn.read_sensor(), &status_bar_item, show_charging)
                            .emit(&app)
                            .unwrap();
                    },
                    SenderMessage::StatusBarShowCharging(show) => {
                        show_charging = show;
                        PowerUpdatedEvent::new_with(&smc_conn.read_sensor(), &status_bar_item, show_charging)
                            .emit(&app)
                            .unwrap();
                    },
                    SenderMessage::WindowVisibilityChanged(visible) => {
                        any_window_visible = visible;
                        // Send immediate update when a window becomes visible
                        if visible {
                            let smc = smc_conn.read_sensor();
                            PowerTickEvent {
                                data: (&get_mac_ioreg().unwrap(), &smc).into(),
                            }.emit(&app).unwrap();
                            last_system_total = smc.system_total;
                            last_battery_level = smc.current_capacity;
                            last_charging = smc.is_charging();
                        }
                    }
                }
            }
        }
    })
}

pub fn setup_sender_with_events<R: Runtime>(app: &impl Manager<R>) -> mpsc::Sender<SenderMessage> {
    let app = app.app_handle();
    let (sender_tx, rx) = mpsc::channel(10);
    start_sender(app, rx);

    // send an immediate update and mark visible when the main window is loaded
    let tx = sender_tx.clone();
    WindowLoadedEvent::listen(app, move |_| {
        let tx = tx.clone();
        async_runtime::spawn(async move {
            let _ = tx.send(SenderMessage::WindowVisibilityChanged(true)).await;
            let _ = tx.send(SenderMessage::ImmediateSend).await;
        });
    });

    let tx = sender_tx.clone();
    PreferenceEvent::listen(app, move |event| {
        if let Some(msg) = match event.payload {
            PreferenceEvent::UpdateInterval(interval) => Some(SenderMessage::ChangeInterval(
                Duration::from_millis(interval.into()),
            )),
            PreferenceEvent::StatusBarItem(item) => Some(SenderMessage::ChangeStatusBarItem(item)),
            PreferenceEvent::StatusBarShowCharging(show) => {
                Some(SenderMessage::StatusBarShowCharging(show))
            }
            PreferenceEvent::Language(_) => {
                // No need to send, perform some menu refreshing
                None
            }
            _ => None,
        } {
            let tx = tx.clone();
            async_runtime::spawn(async move {
                tx.send(msg).await.unwrap();
            });
        }
    });

    sender_tx
}
