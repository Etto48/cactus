use dioxus::desktop::LogicalSize;
pub use dioxus::prelude::*;

use crate::{app::{log::{log_to_component, Log}, side_panel::side_panel_contents}, commands::commands::parse_command, connection::{connection_manager::ConnectionManager, connection_map::ConnectionMap}};

pub fn app() -> Element {
    dioxus::desktop::window().set_title("🌵Cactus");
    dioxus::desktop::window().set_min_inner_size(Some(LogicalSize::new(600.0, 400.0)));
    static CSS: Asset = asset!("/assets/style.css");
    static ENTER_SVG: Asset = asset!("/assets/arrow-return-left.svg");
    let log = use_signal_sync(|| {
        let mut log = Log::default();
        log.log_i("Cactus started");
        log
    });
    let connection_map = use_signal_sync(|| {
        ConnectionMap::default()
    });
    let connection_manager = use_signal_sync(|| {
        ConnectionManager::new(log, connection_map)
    });
    let mut input_string = use_signal_sync(|| String::new());
    let mut enter_handler = move || {
        if let Ok(mut input_string) = input_string.try_write() {
            if input_string.is_empty() {
                return;
            }
            parse_command(input_string.clone(), connection_manager, log);
            input_string.clear();
        }
    };
    let last_log_message = use_signal(|| None::<Event<MountedData>>);
    use_effect(move || {
        if let Some(message) = last_log_message() {
            let _ = message.scroll_to(ScrollBehavior::Smooth);
        }
    });
    rsx!{
        document::Stylesheet{ href: CSS }
        div {
            class: "container",
            div {
                class: "side-panel",
                side_panel_contents { connection_manager, log }
            }
            div {
                class: "central-container",
                div {
                    class: "main-panel",
                    log_to_component { log, last_log_message }
                }
                div {
                    class: "input-panel",
                    div {
                        class: "input-wrapper",
                        input {
                            class: "input-field",
                            placeholder: "Type here...",
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