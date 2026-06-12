use std::process;
use std::time::{Duration, Instant};

use tauri::{
    async_runtime,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent},
    ActivationPolicy, Manager, Runtime,
};
use tauri_plugin_nspopover::{AppExt, WindowExt as _};
use tauri_specta::Event;
use tokio::sync::mpsc;

use crate::{event::PowerUpdatedEvent, ext::WebviewWindowExt, local::SenderMessage};

pub fn setup_tray_icon<R: Runtime>(app: &impl Manager<R>) -> tauri::Result<()> {
    let show = MenuItemBuilder::new("Show Window").build(app)?;
    let quit = MenuItemBuilder::new("Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .separator()
        .item(&quit)
        .build()
        .unwrap();

    let tray_icon = TrayIconBuilder::with_id("main")
        .title("0 w")
        .menu_on_left_click(false)
        .menu(&menu)
        .build(app)
        .unwrap();

    tray_icon.on_menu_event(move |tray_handle, event| match event.id() {
        val if val == show.id() => {
            let (window, _) = tray_handle
                .app_handle()
                .get_or_create_window("main")
                .unwrap();

            if !window.is_visible().unwrap() {
                window.show().unwrap();
                window.set_focus().unwrap();

                tray_handle
                    .app_handle()
                    .set_activation_policy(ActivationPolicy::Regular)
                    .unwrap();
            }
        }
        val if val == quit.id() => {
            tray_handle.app_handle().cleanup_before_exit();
            process::exit(0);
        }
        _ => {}
    });

    tray_icon.on_tray_icon_event(move |tray_handle, event| {
        tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &event);
        if let TrayIconEvent::Click {
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let handle = tray_handle.app_handle();
            let showing = !handle.is_popover_shown();
            if showing {
                handle.show_popover();
            } else {
                handle.hide_popover();
            }
            // Notify sender of popover visibility change
            if let Some(tx) = handle.try_state::<mpsc::Sender<SenderMessage>>() {
                let tx = tx.inner().clone();
                async_runtime::spawn(async move {
                    let _ = tx.send(SenderMessage::WindowVisibilityChanged(showing)).await;
                });
            }
        }
    });

    // Throttle tray icon updates: only call set_title when the displayed text
    // changes OR at least 5 seconds have elapsed. This avoids redundant menu bar
    // redraws that spam WindowServer with state change notifications.
    use std::sync::Mutex;
    struct TrayState {
        last_title: String,
        last_update: Instant,
    }
    let state = std::sync::Arc::new(Mutex::new(TrayState {
        last_title: String::from("0 w"),
        last_update: Instant::now(),
    }));
    let max_stale = Duration::from_secs(5);
    PowerUpdatedEvent::listen(app.app_handle(), move |event| {
        let new_title = &event.payload.0;
        let mut s = state.lock().unwrap();
        let elapsed = s.last_update.elapsed();
        if s.last_title != *new_title || elapsed >= max_stale {
            tray_icon.set_title(Some(new_title)).unwrap();
            s.last_title = new_title.clone();
            s.last_update = Instant::now();
        }
    });

    app.popover_window().unwrap().to_popover();

    Ok(())
}
