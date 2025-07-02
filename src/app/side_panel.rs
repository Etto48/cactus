use std::net::SocketAddr;

use dioxus::prelude::*;

use crate::{app::{context_menu::{ContextMenu, ContextMenuContent, ContextMenuInfo}, log::Log}, connection::{chats::Chats, connection_manager::ConnectionManager}};

#[component]
pub fn side_panel_contents(
    connection_manager: SyncSignal<ConnectionManager>, 
    log: SyncSignal<Log>, 
    active_chat: SyncSignal<Option<(String, SocketAddr)>>,
    chats: SyncSignal<Chats>,
    context_menu: Signal<ContextMenu>,
) -> Element {
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
            class: "side-panel-item system ".to_owned() + if active_chat().is_none() {"active"} else { "" },
            onclick: move |_| {
                active_chat.set(None);
            },
            oncontextmenu: move |e| {
                context_menu.set(Some(ContextMenuInfo::new(
                    ContextMenuContent::System, &e
                )))
            },
            div {
                class: "side-panel-item-wrapper",
                span {
                    class: "connection-name",
                    "System"
                }
                div {
                    class: "message-preview-wrapper",
                    div {
                        class: "message-preview-wrapper",
                        if let Some(notification) = log.read().notification {
                            div {
                                class: "notification-dot {notification.to_string().to_lowercase()}",
                                aria_label: "New message notification",
                            }
                        }
                        if let Some(last_message) = log.read().get_last_message() {
                            span {
                                class: "message-preview",
                                "{last_message}"
                            }
                        } else {
                            span {
                                class: "message-preview empty",
                                "No messages"
                            }
                        }
                    }
                }
            }
        }
        for connection in connections_copy.drain(..) {
            div {
                class: "side-panel-item ".to_owned() + if let Some((_, addr)) = active_chat() { if addr == connection.1 { "active" } else { "" } } else { "" },
                onclick: move |_| {
                    active_chat.set(Some((connection.0.clone(), connection.1.clone())));
                },
                oncontextmenu: move |e| {
                    context_menu.set(Some(ContextMenuInfo::new(
                        ContextMenuContent::Connection {
                            address: connection.1.clone(),
                        }, &e
                    )))
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
                    div {
                        class: "message-preview-wrapper",
                        if let Some(messages) = chats.read().get_messages(&connection.1) {
                            if let Some(last_message) = messages.last() {
                                if messages.notification {
                                    div {
                                        class: "notification-dot",
                                        aria_label: "New message notification",
                                    }
                                }
                                span {
                                    class: "message-preview",
                                    if last_message.direction.is_received() {
                                        "{last_message.content}"
                                    } else {
                                        "You: {last_message.content}"
                                    }
                                }
                            } else {
                                span {
                                    class: "message-preview empty",
                                    "No messages"
                                }
                            } 
                        } else {
                            span {
                                class: "message-preview empty",
                                "No messages"
                            }
                        }
                    }
                }
            }
        }
    }
}