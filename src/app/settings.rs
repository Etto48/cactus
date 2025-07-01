use dioxus::prelude::*;

use crate::app::log::Log;

#[component]
pub fn settings_component(username: SyncSignal<String>, log: SyncSignal<Log>, show_settings: Signal<bool>) -> Element {
    static X_SVG: Asset = asset!("/assets/x.svg");
    rsx! {
        div {
            class: "settings-background",
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    show_settings.set(false);
                }
            },
            onclick: move |_| {
                show_settings.set(false);
            },
            div {
                class: "settings",
                onclick: move |e| {
                    e.stop_propagation();
                },
                div {
                    class: "settings-header",
                    span {
                        "Settings"
                    }
                    button {
                        class: "close-button",
                        onclick: move |_| {
                            show_settings.set(false);
                        },
                        img {
                            src: X_SVG,
                            alt: "Close Settings",
                        }
                    }
                }
                div {
                    class: "settings-body",
                    label {
                        for: "username-input",
                        "Username:"
                    }
                    input {
                        id: "username-input",
                        value: "{username}",
                        oninput: move |e| username.set(e.value().clone())
                    }
                    label {
                        for: "log-level-select",
                        "Log Level:"
                    }
                    div {
                        class: "select-wrapper",
                        select {
                            id: "log-level-select",
                            value: log.read().level.to_string(),
                            onchange: move |e| {
                                if let Ok(level) = e.value().parse::<crate::app::log::LogLevel>() {
                                    log.write().level = level;
                                }
                            },
                            option {
                                value: "DEBUG",
                                "Debug"
                            }
                            option {
                                value: "INFO",
                                "Info"
                            }
                            option {
                                value: "WARNING",
                                "Warning"
                            }
                            option {
                                value: "ERROR",
                                "Error"
                            }
                        }
                    }
                }
            }
        }
    }
}