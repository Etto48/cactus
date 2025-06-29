use std::{io::{Read, Write}, net::{SocketAddr, TcpStream}, sync::{atomic::AtomicBool, Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};

use dioxus::signals::{Readable, SyncSignal, Writable};

use crate::{app::log::Log, connection::{chats::{Chats, MessageDirection}, connection_map::ConnectionMap, message::Message}};

pub struct Connection {
    pub name: Arc<Mutex<Option<String>>>,
    pub address: SocketAddr,
    pub socket: TcpStream,
    pub running: Arc<AtomicBool>,
    pub thread: Option<JoinHandle<()>>,
    pub log: SyncSignal<Log>,
    pub chats: SyncSignal<Chats>,
}

impl Connection {
    pub fn new(
        address: SocketAddr, 
        socket: TcpStream, 
        mut log: SyncSignal<Log>, 
        connection_map: SyncSignal<ConnectionMap>, 
        chats: SyncSignal<Chats>,
        username: SyncSignal<String>,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let name = Arc::new(Mutex::new(None));
        socket.set_nonblocking(true).expect("Failed to set socket to non-blocking");
            
        let thread = Some(std::thread::spawn({
            let running = running.clone();
            let socket = socket.try_clone().expect("Failed to clone socket");
            let name = name.clone();
            move || Self::run(running, socket,address,  name, log.clone(), connection_map, chats.clone())
        }));
        
        let mut ret = Connection {
            name,
            address,
            socket,
            running,
            thread,
            log,
            chats,
        };

        {
            let username = username.read();
            if !username.is_empty() {
                if let Err(e) = ret.send(Message::Hello(username.clone())) {
                    log.write().log_e(format!("Failed to send hello message to {}: {:?}", address, e));
                }       
            }
        }

        ret
    }

    fn run(
        running: Arc<AtomicBool>, 
        mut socket: TcpStream, 
        address: SocketAddr, 
        name: Arc<Mutex<Option<String>>>, 
        mut log: SyncSignal<Log>, 
        mut connection_map: SyncSignal<ConnectionMap>,
        mut chats: SyncSignal<Chats>,
    ) {
        let mut receive_buffer = [0; 1024*4];
        let mut expected_len = None;
        let mut receive_index = 0;
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            match socket.read(&mut receive_buffer[receive_index..]) {
                Ok(0) => {
                    break;
                },
                Ok(n) => {
                    receive_index += n;
                    if expected_len.is_none() && receive_index >= 8 {
                        expected_len = Some(u64::from_le_bytes(receive_buffer[0..8].try_into().unwrap()) as usize);
                    }
                    while let Some(len) = expected_len {
                        if receive_index >= len + 8 {
                            let message = Message::deserialize(&receive_buffer);
                            if let Some(msg) = message {
                                match msg {
                                    Message::Hello(n) => {
                                        if !n.is_empty() {
                                            connection_map.write().rename_connection(address, n.clone());
                                            log.write().log_d(format!("{} is now called {}", 
                                                address, 
                                                n));
                                        }
                                    },
                                    Message::Text(text) => {
                                        log.write().log_d(format!("Received text message from {}: {}", 
                                            name.lock().unwrap().as_ref().unwrap_or(&format!("{}", address)), text));
                                        chats.write().add_message(address, MessageDirection::Received, text.clone());
                                    }
                                }
                            }
                            receive_buffer.rotate_left(8 + len);
                            receive_index -= 8 + len;
                            if receive_index >= 8 {
                                expected_len = Some(u64::from_le_bytes(receive_buffer[0..8].try_into().unwrap()) as usize);
                            } else {
                                expected_len = None;
                            }
                        } else {
                            break; // Not enough data for the next message
                        }
                    }
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut{
                        thread::sleep(Duration::from_millis(50)); // Sleep to avoid busy waiting
                    } else {
                        log.write().log_e(format!("Error reading from socket {}: {:?}", address, e));
                        break;
                    }
                }
            }
        }
        running.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_name(&self) -> String {
        self.name.lock().unwrap().clone().unwrap_or_else(|| format!("{}", self.address))
    }

    pub fn send(&mut self, message: Message) -> std::io::Result<()> {
        if let Message::Text(text) = &message {
            self.chats.write().add_message(self.address, MessageDirection::Sent, text.clone());
        }
        let serialized = message.serialize();
        self.socket.write_all(&serialized)?;
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        let name = self.get_name();
        let mut log = self.log.try_write().ok();
        if let Ok(mut chats) = self.chats.try_write() {
            chats.clear_chat(&self.address)
        }
        if let Err(e) = self.socket.shutdown(std::net::Shutdown::Both) {
            if let Some(log) = &mut log {
                log.log_e(format!("Failed to shutdown socket {}: {:?}", self.address, e));
            }
        }
        if let Err(e) = std::mem::take(&mut self.thread).unwrap().join() {
            if let Some(log) = &mut log {
                log.log_e(format!("Failed to join connection thread for {}: {:?}", self.address, e));
            }
        }
        if let Some(log) = &mut log {
            log.log_d(format!("Connection to {} closed", name));
        }
    }
}