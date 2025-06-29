use std::net::SocketAddr;

use dioxus::prelude::*;

use crate::connection::{chats::Chats, connection_manager::ConnectionManager};

#[component]
pub fn chat_to_component(
    connection_manager: SyncSignal<ConnectionManager>, 
    active_chat: ReadOnlySignal<(String, SocketAddr)>,
    chats: SyncSignal<Chats>, 
    mut last_message: Signal<Option<Event<MountedData>>>,
) -> Element {
    let (name, address) = active_chat();
    if let Some(messages) = chats.read().get_messages(&address) {
        let messages_len = messages.len();
        rsx! {
            for (i, message) in messages.iter().enumerate() {
                div {
                    class: "chat-message {message.direction.to_str()}",
                    onmounted: move |e| {
                        if i == messages_len - 1 {
                            last_message.set(Some(e));
                            println!("Last message set");
                        }
                    },
                    div {
                        class: "chat-message-wrapper",
                        if message.direction.is_received() {
                            span {
                                class: "chat-message-source",
                                "{name}"
                            }
                        }
                        span {
                            class: "chat-message-content",
                            "{message.content}"    
                        }
                    }
                    span {
                        class: "chat-message-timestamp",
                        "{message.fmt_timestamp()}"
                    }
                }
            }
        }
    } else { rsx! { } }
}