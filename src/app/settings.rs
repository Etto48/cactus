use dioxus::prelude::*;

use crate::app::log::Log;

#[component]
pub fn settings_component(username: SyncSignal<String>, log: SyncSignal<Log>, show_settings: Signal<bool>) -> Element {
    static X_SVG: Asset = asset!("/assets/x.svg");
    rsx! {
        div {
            class: "settings-background",
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
                }
            }
        }
    }
}