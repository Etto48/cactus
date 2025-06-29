use std::net::SocketAddr;

use dioxus::prelude::*;

use crate::{app::log::Log, connection::connection_manager::ConnectionManager};

#[component]
pub fn side_panel_contents(
    connection_manager: SyncSignal<ConnectionManager>, 
    log: SyncSignal<Log>, 
    active_chat: Signal<Option<(String, SocketAddr)>>,
) -> Element {
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
            class: "side-panel-item ".to_owned() + if active_chat().is_none() {"active"} else { "" },
            onclick: move |_| {
                active_chat.set(None);
            },
            "System"
        }
        for connection in connections_copy.drain(..) {
            div {
                class: "side-panel-item ".to_owned() + if let Some((_, addr)) = active_chat() { if addr == connection.1 { "active" } else { "" } } else { "" },
                onclick: move |_| {
                    active_chat.set(Some((connection.0.clone(), connection.1.clone())));
                },
                div {
                    class: "side-panel-item-wrapper",
                    span {
                        class: "connection-name",
                        "{connection.0}"
                    }
                    span {
                        class: "connection-address",
                        "{connection.1}"
                    }
                }
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