use std::{io::{Read, Write}, net::{SocketAddr, TcpStream}, sync::{atomic::AtomicBool, Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};

use dioxus::signals::{Readable, SyncSignal, Writable};
use snow::{HandshakeState, TransportState};

use crate::{app::log::Log, connection::{chats::{Chats, MessageDirection}, connection_manager::EncryptionInfo, connection_map::ConnectionMap, message::Message}};

pub struct Connection {
    pub name: Arc<Mutex<Option<String>>>,
    pub address: SocketAddr,
    pub socket: TcpStream,
    pub running: Arc<AtomicBool>,
    pub thread: Option<JoinHandle<()>>,
    pub log: SyncSignal<Log>,
    pub chats: SyncSignal<Chats>,
    pub encryption_info: Arc<EncryptionInfo>,
    pub transport_state: Arc<Mutex<Option<TransportState>>>,
}

impl Connection {
    pub fn new(
        address: SocketAddr, 
        socket: TcpStream, 
        mut log: SyncSignal<Log>, 
        connection_map: SyncSignal<ConnectionMap>, 
        chats: SyncSignal<Chats>,
        username: SyncSignal<String>,
        encryption_info: Arc<EncryptionInfo>,
        is_initiator: bool,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let name = Arc::new(Mutex::new(None));
        socket.set_nonblocking(true).expect("Failed to set socket to non-blocking");
        
        let noise_builder = encryption_info.get_builder();
        let handshake_state = if is_initiator {
            noise_builder.build_initiator().expect("Failed to build initiator")
        } else {
            noise_builder.build_responder().expect("Failed to build responder")
        };
        let handshake_state = handshake_state;
        let transport_state = Arc::new(Mutex::new(None));

        let thread = Some(std::thread::spawn({
            let running = running.clone();
            let socket = socket.try_clone().expect("Failed to clone socket");
            let name = name.clone();
            let transport_state = transport_state.clone();
            move || Self::run(running, socket,address,  name, log.clone(), connection_map, chats.clone(), handshake_state, transport_state)
        }));
        
        let mut ret = Connection {
            name,
            address,
            socket,
            running,
            thread,
            log,
            chats,
            encryption_info,
            transport_state,
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
        handshake_state: HandshakeState,
        transport_state: Arc<Mutex<Option<TransportState>>>,
    ) {
        let mut receive_buffer = [0u8; 65536]; // 64KB buffer for receiving
        let mut payload_buffer = [0u8; 65536]; // 64KB buffer for decrypted payload
        let mut expected_len = None;
        let mut receive_index = 0;
        let mut handshake_state = Some(handshake_state);
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(handshake) = handshake_state.as_mut() {
                let mut send_buffer = [0u8; 65536]; // 64KB buffer for sending;
                if handshake.is_handshake_finished() {
                    let mut transport_state = transport_state.lock().unwrap();
                    
                    *transport_state = Some(
                        std::mem::take(&mut handshake_state).unwrap().into_transport_mode().expect("Failed to switch to transport mode"));
                    continue;
                }
                if handshake.is_my_turn() {
                    let len = handshake.write_message(&[], &mut send_buffer).expect("Failed to write handshake message");
                    match socket.write_all(&send_buffer[..len]) {
                        Ok(()) => {},
                        Err(e) => {
                            log.write().log_e(format!("Error during handshake with {}: {:?}", address, e));
                        },
                    }
                } else {
                    match socket.read(&mut receive_buffer) {
                        Ok(0) => {
                            log.write().log_d(format!("Connection to {} closed during handshake", address));
                            break;
                        },
                        Ok(n) => {
                            if n > 0 {
                                if let Err(e) = handshake.read_message(&receive_buffer[..n], &mut payload_buffer) {
                                    log.write().log_e(format!("Error reading handshake message from {}: {:?}", address, e));
                                }
                            }
                        },
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut {
                                thread::sleep(Duration::from_millis(50)); // Sleep to avoid busy waiting
                            } else {
                                log.write().log_e(format!("Error reading from socket {}: {:?}", address, e));
                                break;
                            }
                        }
                    }
                }
            } else {
                match socket.read(&mut receive_buffer) {
                    Ok(0) => {
                        break;
                    },
                    Ok(n) => {
                        let mut transport_lock = transport_state.lock().unwrap();
                        let transport = transport_lock.as_mut().unwrap();
                        match transport.read_message(&receive_buffer[..n], &mut payload_buffer) {
                            Ok(n) => {
                                drop(transport_lock);
                                receive_index += n;
                                if expected_len.is_none() && receive_index >= 8 {
                                    let len = u64::from_le_bytes(payload_buffer[0..8].try_into().unwrap()) as usize;
                                    if len > payload_buffer.len() - 8 {
                                        log.write().log_e(format!("Received message length {} exceeds buffer size", len));
                                        break;
                                    }
                                    expected_len = Some(len);
                                }
                                while let Some(len) = expected_len {
                                    if receive_index >= len + 8 {
                                        let message = Message::deserialize(&payload_buffer);
                                        if let Some(msg) = message {
                                            match msg {
                                                Message::Hello(n) => {
                                                    if !n.is_empty() {
                                                        while !connection_map.write().rename_connection(address, n.clone()) {
                                                            // Wait until the connection is in the connection map
                                                            thread::sleep(Duration::from_millis(100));
                                                        }
                                                        
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
                                        payload_buffer.rotate_left(8 + len);
                                        receive_index -= 8 + len;
                                        if receive_index >= 8 {
                                            expected_len = Some(u64::from_le_bytes(payload_buffer[0..8].try_into().unwrap()) as usize);
                                        } else {
                                            expected_len = None;
                                        }
                                    } else {
                                        break; // Not enough data for the next message
                                    }
                                }
                            },
                            Err(e) => {
                                log.write().log_e(format!("Error decrypting message from {}: {:?}", address, e));
                                break;
                            },
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
        }
        running.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get_name(&self) -> String {
        self.name.lock().unwrap().clone().unwrap_or_else(|| format!("{}", self.address))
    }

    pub fn send(&mut self, message: Message) -> std::io::Result<()> {
        let mut send_buffer = [0u8; 65536]; // 64KB buffer for sending
        if let Message::Text(text) = &message {
            self.chats.write().add_message(self.address, MessageDirection::Sent, text.clone());
        }
        let serialized = message.serialize();
        loop {
            let mut transport_lock = self.transport_state.lock().unwrap();
            if let Some(transport) = transport_lock.as_mut() {
                let len = transport.write_message(&serialized, &mut send_buffer)
                    .expect("Failed to write message");
                self.socket.write_all(&send_buffer[..len])?;
                break;
            } else {
                drop(transport_lock); // Drop the lock to avoid deadlock
                thread::sleep(Duration::from_millis(50)); // Wait for handshake to finish
                continue;
            }
        }
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