use std::{collections::HashMap, net::SocketAddr};

use dioxus::prelude::*;

use crate::connection::{chats::{ChatMessage, Chats}, connection_manager::ConnectionManager};

#[component]
pub fn chat_to_component(
    connection_manager: SyncSignal<ConnectionManager>, 
    active_chat: ReadOnlySignal<(String, SocketAddr)>,
    chats: SyncSignal<Chats>, 
    mut last_message_ref: Signal<Option<Event<MountedData>>>,
    mut message_refs: Signal<HashMap<usize, Event<MountedData>>>,
) -> Element {
    let (name, address) = active_chat();
    if let Some(messages) = chats.read().get_messages(&address) {
        let messages_len = messages.len();
        use_effect(move || {
            active_chat.read();
            if messages_len > 0 {
                if let Some(last_message) = message_refs.read().get(&(messages_len - 1)) {
                    last_message_ref.set(Some(last_message.clone()));
                }
            }
        });
        rsx! {
            for (i, message) in messages.iter().enumerate() {
                div {
                    class: "chat-message ".to_owned() + message.direction.to_str(),
                    div {
                        class: "chat-message-wrapper",
                        onmounted: move |e| {
                            message_refs.write().insert(i, e);
                            chats.write().reset_notification(&address);
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

#[component]
fn chat_message_component(
    mut message_refs: Signal<HashMap<usize, Event<MountedData>>>,
    message: ReadOnlySignal<ChatMessage>,
    name: ReadOnlySignal<String>,
    i: ReadOnlySignal<usize>
) -> Element {
    rsx! {
        
    }
}