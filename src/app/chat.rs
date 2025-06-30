use std::net::SocketAddr;

use dioxus::prelude::*;

use crate::connection::{chats::Chats, connection_manager::ConnectionManager};

#[component]
pub fn chat_to_component(
    connection_manager: SyncSignal<ConnectionManager>, 
    active_chat: ReadOnlySignal<(String, SocketAddr)>,
    chats: SyncSignal<Chats>, 
    mut message_refs: Signal<Vec<Event<MountedData>>>,
    mut last_message_index: Signal<Option<usize>>,
) -> Element {
    let (name, address) = active_chat();
    if let Some(messages) = chats.read().get_messages(&address) {
        let messages_len = messages.len();
        use_effect(move || {
            active_chat.read();
            last_message_index.set(Some(messages_len - 1));
        });
        rsx! {
            for message in messages.iter() {
                div {
                    class: "chat-message ".to_owned() + message.direction.to_str(),
                    div {
                        class: "chat-message-wrapper",
                        onmounted: move |e| {
                            message_refs.write().push(e);
                            last_message_index.set(Some(messages_len - 1));
                        },
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