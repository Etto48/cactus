use std::{net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpListener, TcpStream}, sync::{atomic::AtomicBool, Arc}, thread::{self, JoinHandle}};

use dioxus::signals::{Readable, SyncSignal, Writable};
use snow::{Builder, Keypair};

use crate::{app::log::Log, connection::{chats::Chats, connection_map::ConnectionMap}};

pub struct EncryptionInfo {
    pub static_keypair: Keypair,
    pub args: String,
}

impl Clone for EncryptionInfo {
    fn clone(&self) -> Self {
        EncryptionInfo {
            static_keypair: Keypair { 
                private: self.static_keypair.private.clone(), 
                public: self.static_keypair.public.clone() 
            },
            args: self.args.clone(),
        }
    }
}

impl EncryptionInfo {
    pub fn new(args: impl Into<String>) -> Self {
        let args = args.into();
        let builder = Builder::new(args.parse().expect("Failed to parse noise protocol arguments"));
        let static_keypair = builder.generate_keypair().expect("Failed to generate keypair");

        EncryptionInfo { static_keypair, args }
    }

    pub fn get_builder(&self) -> Builder {
        Builder::new(self.args.parse().expect("Failed to parse noise protocol arguments"))
            .local_private_key(&self.static_keypair.private)
    }
}

pub struct ConnectionManager {
    pub connections: SyncSignal<ConnectionMap>,
    username: SyncSignal<String>,
    log: SyncSignal<Log>,
    chats: SyncSignal<Chats>,
    running: Arc<AtomicBool>,
    encryption_info: Arc<EncryptionInfo>,
    thread: Option<JoinHandle<()>>,
}

impl ConnectionManager {
    pub fn new(
        log: SyncSignal<Log>, 
        connection_map: SyncSignal<ConnectionMap>, 
        chats: SyncSignal<Chats>,
        username: SyncSignal<String>,
    ) -> Self {
        let connections = connection_map;
        let running = Arc::new(AtomicBool::new(true));
        let listener = TcpListener::bind(
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 4848, 0, 0)))
            .expect("Failed to bind TCP listener");
        listener.set_nonblocking(true).expect("Failed to set listener to non-blocking");

        let noise_args = "Noise_XX_25519_ChaChaPoly_BLAKE2b";
        let encryption_info = EncryptionInfo::new(noise_args);
        let encryption_info = Arc::new(encryption_info);

        let thread = Some(std::thread::spawn({
            let running = running.clone();
            let connections = connections.clone();
            let log = log.clone();
            let chats = chats.clone();
            let username = username.clone();
            let encryption_info = encryption_info.clone();
            move || Self::run(running, listener, log, connections, chats, username, encryption_info)
        }));
        ConnectionManager {
            connections,
            username,
            log,
            chats,
            running,
            encryption_info,
            thread,
        }
    }
    fn run(
        running: Arc<AtomicBool>, 
        listener: TcpListener, 
        mut log: SyncSignal<Log>, 
        mut connections: SyncSignal<ConnectionMap>, 
        chats: SyncSignal<Chats>,
        username: SyncSignal<String>,
        encryption_info: Arc<EncryptionInfo>,
    ) {
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            match listener.accept() {
                Ok((socket, address)) => {
                    log.write().log_i(format!("Accepted connection from {}", address));
                    let connection = crate::connection::connection::Connection::new(
                        address,
                        socket,
                        log.clone(),
                        connections.clone(),
                        chats.clone(),
                        username.clone(),
                        encryption_info.clone(),
                        false,
                    );
                    connections.write().add(connection);
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        thread::sleep(std::time::Duration::from_millis(100)); // Avoid busy waiting
                    } else {
                        log.write().log_e(format!("Failed to accept connection: {}", e));
                    }
                },
            }
            if let Ok(mut conns) = connections.try_write() {
                let mut dead_addrs = Vec::new();
            
                for connection in conns.iter() {
                    if !connection.running.load(std::sync::atomic::Ordering::SeqCst) {
                        dead_addrs.push(connection.address);
                    }
                }
                for address in dead_addrs {
                    conns.remove_by_address(&address);
                }
            }
        }
    }

    pub fn connect(&mut self, address: SocketAddr) -> std::io::Result<()>{
        if self.connections.read().get_by_address(&address).is_some() {
            self.log.write().log_w(format!("Already connected to {}", address));
            return Ok(());
        }
        let socket = TcpStream::connect(address)?;
        self.log.write().log_i(format!("Connected to {}", address));
        let connection = crate::connection::connection::Connection::new(
            address,
            socket,
            self.log.clone(),
            self.connections.clone(),
            self.chats.clone(),
            self.username.clone(),
            self.encryption_info.clone(),
            true,
        );
        self.connections.write().add(connection);
        Ok(())
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            if let Err(e) = thread.join() {
                eprintln!("Failed to join connection manager thread: {:?}", e);
            }
        }
    }
}