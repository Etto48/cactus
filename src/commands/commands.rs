use std::net::ToSocketAddrs;

use dioxus::signals::{SyncSignal, Writable};

use crate::{app::log::Log, connection::connection_manager::ConnectionManager};

pub fn parse_command(command: String, mut connection_manager: SyncSignal<ConnectionManager>, mut log: SyncSignal<Log>) {
    let command = command.trim();
    if command.is_empty() {
        return;
    }
    let mut args = Vec::new();
    let mut new_token = String::new();
    let mut next_is_escape = false;
    for c in command.chars() {
        if c == '\\' && !next_is_escape {
            next_is_escape = true;
        } else if c.is_whitespace() && !next_is_escape {
            if !new_token.is_empty() {
                args.push(std::mem::take(&mut new_token));
            }
        } else {
            next_is_escape = false;
            new_token.push(c);
        }
    }
    if !new_token.is_empty() {
        args.push(new_token);
    }

    match args[0].to_lowercase().as_str() {
        "connect" => {
            if args.len() < 2 {
                log.write().log_e("Usage: connect <address>".to_string());
                return;
            }
            if let Ok(addr) = args[1].parse::<std::net::SocketAddr>() {
                if let Err(e) = connection_manager.write().connect(addr) {
                    log.write().log_e(format!("Failed to connect to {}: {}", addr, e));
                }
            } else if let Ok(addrs) = args[1].to_socket_addrs() {
                let connected = 'try_all_addrs: {
                    for addr in addrs {
                        if connection_manager.write().connect(addr).is_ok() {
                            break 'try_all_addrs true;
                        }
                    }
                    false
                };
                if !connected {
                    log.write().log_e(format!("Failed to connect to {}", args[1]));
                }
            } else {
                log.write().log_e(format!("Invalid address: {}", args[1]));
            }
        }
        "disconnect" => {
            if args.len() < 2 {
                log.write().log_e("Usage: disconnect <name>".to_string());
                return;
            }
            if let Ok(mut connection_manager) = connection_manager.try_write() {
                if let Ok(mut connections) = connection_manager.connections.try_write() {
                    if connections.remove_by_any(&args[1]).is_some() {
                        log.write().log_d(format!("Disconnected from {}", args[1]));
                    } else {
                        log.write().log_e(format!("No connection found with name or address: {}", args[1]));
                    }
                }
            }
        },
        "send" => {
            if args.len() < 3 {
                log.write().log_e("Usage: send <destination> <kind> [<contents> ...]");
                return;
            }
            let destination = &args[1];
            let kind = &args[2];
            let message = match kind.to_lowercase().as_str() {
                "hello" => {
                    if args.len() < 4 {
                        log.write().log_e("Usage: send <destination> hello <name>");
                        return;
                    }
                    let name = args[3].to_string();
                    crate::connection::message::Message::Hello(name)
                }
                "text" => {
                    if args.len() < 4 {
                        log.write().log_e("Usage: send <destination> text {<contents> ...}");
                        return;
                    }
                    let contents = args[3..].join(" ");
                    crate::connection::message::Message::Text(contents)
                }
                _ => {
                    log.write().log_e(format!("Unknown message kind: {}", kind));
                    return;
                }
            };

            if let Ok(mut connection_manager) = connection_manager.try_write() {
                if let Ok(mut connections) = connection_manager.connections.try_write() {
                    if let Some(connection) = connections.get_mut_by_any(&destination) {
                        if let Err(e) = connection.send(message) {
                            log.write().log_e(format!("Failed to send message to {}: {}", destination, e));
                        } else {
                            log.write().log_d(format!("Message sent to {}", destination));
                        }
                    } else {
                        log.write().log_e(format!("No connection found with name or address: {}", destination));
                        return;
                    }
                }
            }
        }
        _ => {
            log.write().log_e(format!("Unknown command: {}", command));
        }
    }
}