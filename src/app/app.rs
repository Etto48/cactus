use std::{collections::HashMap, net::SocketAddr};

use dioxus::desktop::{tao::window::Icon, LogicalSize};
pub use dioxus::prelude::*;

use crate::{app::{chat::chat_to_component, log::{log_to_component, Log}, settings::settings_component, side_panel::side_panel_contents}, commands::commands::parse_command, connection::{chats::Chats, connection_manager::ConnectionManager, connection_map::ConnectionMap, message::Message}};

pub fn app() -> Element {
    static TITLE: &'static str = "🌵Cactus";
    dioxus::desktop::window().set_title(TITLE);
    dioxus::desktop::window().set_min_inner_size(Some(LogicalSize::new(700.0, 400.0)));
    let icon_rgba = include_bytes!("../../assets/icon.rgba");
    dioxus::desktop::window().set_window_icon(Some(Icon::from_rgba(icon_rgba.to_vec(), 32, 32).expect("Failed to create window icon")));

    static CSS: Asset = asset!("/assets/style.css");
    static ENTER_SVG: Asset = asset!("/assets/arrow-return-left.svg");
    static GEAR_SVG: Asset = asset!("/assets/gear-fill.svg");
    static ICON_SVG: Asset = asset!("/assets/icon.svg");
    let mut log = use_signal_sync(|| {
        let mut log = Log::default();
        log.log_i("Cactus started");
        log
    });
    let mut chats = use_signal_sync(|| {
        Chats::default()
    });
    let mut show_settings = use_signal(|| {
        false
    });
    let username = use_signal_sync(|| {
        String::new()
    });
    let active_chat = use_signal_sync(|| {
        None::<(String, SocketAddr)>
    });
    let mut connection_map = use_signal_sync(|| {
        ConnectionMap::new(active_chat)
    });
    let connection_manager = use_signal_sync(|| {
        ConnectionManager::new(log, connection_map, chats, username)
    });
    let mut input_string = use_signal_sync(|| String::new());
    let last_log_message = use_signal(|| None::<Event<MountedData>>);
    let message_refs = use_signal(|| {
        HashMap::<usize, Event<MountedData>>::new()
    });
    let last_message_ref = use_signal(|| None::<Event<MountedData>>);
    let mut enter_handler = move || {
        if let Ok(mut input_string) = input_string.try_write() {
            if input_string.is_empty() {
                return;
            }
            if let Some((name, address)) = active_chat() {
                let message = Message::text(
                    input_string.clone()
                );
                if let Some(connection) = connection_map.write().get_mut_by_address(&address) {
                    if let Err(e) = connection.send(message) {
                        log.write().log_e(format!("Failed to send message to {}: {}", name, e));
                    }
                }
            } else {
                parse_command(input_string.clone(), connection_manager, log);
            }
            input_string.clear();
        }
    };
    use_effect(move || {
        if let Some((_, _)) = active_chat() {
            if let Some(last_message) = last_message_ref() {
                let _ = last_message.scroll_to(ScrollBehavior::Smooth);
            }
        } else {
            if let Some(last_log_message) = last_log_message() {
                let _ = last_log_message.scroll_to(ScrollBehavior::Smooth);
            }
        }
    });
    use_effect(move || {
        if let Some((_, address)) = active_chat() {
            chats.write().reset_notification(&address);    
        } else {
            log.write().reset_notification();
        }
    });
    rsx!{
        document::Link { rel: "icon", href: ICON_SVG, type: "image/svg+xml" }
        document::Title { "{TITLE}" }
        document::Stylesheet{ href: CSS }
        div {
            class: "container",
            oncontextmenu: move |e| {
                e.prevent_default();
            },
            if show_settings() {
                settings_component { username, log, show_settings }
            }
            div {
                class: "side-panel",
                div {
                    class: "side-panel-wrapper",
                    side_panel_contents { connection_manager, log, active_chat, chats }
                }
                div {
                    class: "side-panel-footer",
                    div {
                        class: "side-panel-footer-wrapper",
                        span {
                            class: "username",
                            if username().is_empty() {
                                "Anonymous"
                            } else {
                                "{username}"
                            }
                        }
                    }
                    
                    button {
                        class: "settings-button",
                        onclick: move |_| {
                            show_settings.set(!show_settings());
                        },
                        img {
                            src: GEAR_SVG,
                            alt: "Settings",
                        }
                    }
                }
            }
            div {
                class: "central-container",
                if let Some((name, address)) = active_chat() {
                    div {
                        class: "main-panel-header",
                        span {
                            class: "main-panel-header-name",
                            "{name}"
                        }
                        span {
                            class: "main-panel-header-address",
                            "{address}"
                        }
                    }
                }
                div {
                    class: "main-panel",
                    if let Some(active_chat) = active_chat() {
                        chat_to_component { connection_manager, active_chat, chats, last_message_ref, message_refs }
                    } else {
                        log_to_component { log, last_log_message }
                    }
                }
                div {
                    class: "input-panel",
                    div {
                        class: "input-wrapper",
                        input {
                            class: "input-field",
                            placeholder: if active_chat().is_none() { "Type a command..." } else { "Type a message..." },
                            value: "{input_string}",
                            oninput: move |e| {
                                input_string.set(e.value().clone());
                            },
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    enter_handler();
                                }
                            },
                        }
                        button {
                            class: "input-button",
                            onclick: move |_| {
                                enter_handler();
                            },
                            img {
                                src: ENTER_SVG,
                                alt: "Enter",
                            }
                        }
                    }
                    
                }
            }
        }
    }
}