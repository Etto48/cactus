use dioxus::prelude::*;
use std::net::SocketAddr;

use crate::{app::log::Log, connection::connection_manager::ConnectionManager};

#[derive(Clone, Debug)]
pub enum ContextMenuContent {
    System,
    Connection {
        address: SocketAddr,
    },
    Message {
        address: SocketAddr,
        message_id: usize,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct ContextMenuLocation {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub struct ContextMenuInfo {
    pub content: ContextMenuContent,
    pub location: ContextMenuLocation,
}

pub type ContextMenu = Option<ContextMenuInfo>;

impl ContextMenuInfo {
    pub fn new(content: ContextMenuContent, e: &MouseData) -> Self {
        ContextMenuInfo {
            content,
            location: ContextMenuLocation {
                x: e.client_coordinates().x as f64,
                y: e.client_coordinates().y as f64,
            },
        }
    }
}

#[component]
pub fn context_menu_component(
    context_menu: Signal<ContextMenu>, 
    connection_manager: SyncSignal<ConnectionManager>,
    log: SyncSignal<Log>,
) -> Element {
    static DISCONNECT_SVG: Asset = asset!("/assets/link-slash-solid.svg");
    static CLEAR_LOG_SVG: Asset = asset!("/assets/trash-can-solid.svg");
    if let Some(content) = context_menu() {
        rsx! {
            div {
                class: "context-menu-wrapper",
                onclick: move |_| {
                    context_menu.set(None);
                },
                oncontextmenu: move |_| {
                    context_menu.set(None);
                },
                div {
                    class: "context-menu",
                    style: format!("left: {}px; top: {}px;", content.location.x, content.location.y),
                    match content.content {
                        ContextMenuContent::System => rsx! {
                            button {
                                class: "context-menu-button",
                                onclick: move |_| {
                                    log.write().clear();
                                },
                                img {
                                    class: "context-menu-icon",
                                    src: CLEAR_LOG_SVG,
                                    alt: "Clear log",
                                }
                                "Clear log"
                            }
                        },
                        ContextMenuContent::Connection { address } => rsx! {
                            button {
                                class: "context-menu-button",
                                onclick: move |_| {
                                    connection_manager.write().connections.write().remove_by_address(&address);
                                },
                                img {
                                    class: "context-menu-icon",
                                    src: DISCONNECT_SVG,
                                    alt: "Disconnect",
                                }
                                "Disconnect"
                            }
                        },
                        ContextMenuContent::Message { address, message_id } => rsx! {
                            p { "Message Menu for " }
                            span { "{address}" }
                            p { "Message ID: {message_id}" }
                        },
                    }
                }
            }
        }
    } else {
        return rsx! {};
    }
}