use dioxus::prelude::*;

use crate::{app::log::Log, connection::connection_manager::ConnectionManager};

#[component]
pub fn side_panel_contents(connection_manager: SyncSignal<ConnectionManager>, log: SyncSignal<Log>) -> Element {
    static X_SVG: Asset = asset!("/assets/x.svg");
    let mut connections_copy = Vec::new();
    if let Ok(connection_manager) = connection_manager.try_read() {
        if let Ok(connections) = connection_manager.connections.try_read() {
            for connection in connections.iter() {
                if connection.running.load(std::sync::atomic::Ordering::SeqCst) {
                    connections_copy.push((connection.get_name(), connection.address.clone()))
                }
            }
        }
    }
    rsx!{
        div {
            class: "side-panel-item",
            onclick: move |_| {
                log.write().log_d("Clicked on System");
            },
            "System"
        }
        for connection in connections_copy.drain(..) {
            div {
                class: "side-panel-item",
                onclick: move |_| {
                    log.write().log_d(format!("Clicked on connection: {}", connection.0));
                },
                "{connection.0}"
                button {
                    class: "disconnect-button",
                    onclick: move |e| {
                        e.stop_propagation();
                        connection_manager.write().connections.write().remove_by_address(&connection.1);
                    },
                    img {
                        src: X_SVG,
                        alt: "Disconnect",
                    }
                }
            }
        }
    }
}