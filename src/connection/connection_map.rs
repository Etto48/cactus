use std::{collections::HashMap, net::SocketAddr};

use crate::connection::connection::Connection;

pub struct ConnectionMap {
    connections: HashMap<SocketAddr, Connection>,
    name_to_address: HashMap<String, SocketAddr>,
}

impl Default for ConnectionMap {
    fn default() -> Self {
        ConnectionMap {
            connections: HashMap::new(),
            name_to_address: HashMap::new(),
        }
    }
}

impl ConnectionMap {
    pub fn add(&mut self, connection: Connection) {
        let address = connection.address;
        let name = connection.name.lock().unwrap().clone();
        self.connections.insert(address, connection);
        if let Some(name) = name {
            self.rename_connection(address, name);
        }
    }
    pub fn remove_by_address(&mut self, address: &SocketAddr) -> Option<Connection> {
        if let Some(connection) = self.connections.remove(address) {
            if let Some(name) = connection.name.lock().unwrap().as_ref() {
                self.name_to_address.remove(name);
            }
            return Some(connection);
        }
        None
    }

    pub fn remove_by_name(&mut self, name: &str) -> Option<Connection> {
        if let Some(address) = self.name_to_address.remove(name) {
            return self.remove_by_address(&address);
        }
        None
    }

    pub fn remove_by_any(&mut self, identifier: &str) -> Option<Connection> {
        if let Ok(address) = identifier.parse::<SocketAddr>() {
            return self.remove_by_address(&address);
        } else {
            return self.remove_by_name(identifier);
        }
    }

    pub fn rename_connection(&mut self, address: SocketAddr, mut new_name: String) {
        if let Some(connection) = self.connections.get_mut(&address) {
            let mut name_lock = connection.name.lock().unwrap();
            if let Some(old_name) = name_lock.clone() {
                self.name_to_address.remove(&old_name);
            }
            // Ensure the new name is unique
            let mut counter = 1;
            while self.name_to_address.contains_key(&new_name) {
                // If the name already exists, append a number to make it unique
                new_name = format!("{} ({})", new_name, counter);
                counter += 1;
            }
            // Update the connection's name and the mapping
            *name_lock = Some(new_name.clone());
            self.name_to_address.insert(new_name, address);
        }
    }

    pub fn get_by_address(&self, address: &SocketAddr) -> Option<&Connection> {
        self.connections.get(address)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Connection> {
        if let Some(address) = self.name_to_address.get(name) {
            return self.connections.get(address);
        }
        None
    }

    pub fn get_mut_by_address(&mut self, address: &SocketAddr) -> Option<&mut Connection> {
        self.connections.get_mut(address)
    }

    pub fn get_mut_by_name(&mut self, name: &str) -> Option<&mut Connection> {
        if let Some(address) = self.name_to_address.get(name) {
            return self.connections.get_mut(address);
        }
        None
    }

    pub fn get_by_any(&self, identifier: &str) -> Option<&Connection> {
        if let Ok(address) = identifier.parse::<SocketAddr>() {
            return self.get_by_address(&address);
        } else {
            return self.get_by_name(identifier);
        }
    }

    pub fn get_mut_by_any(&mut self, identifier: &str) -> Option<&mut Connection> {
        if let Ok(address) = identifier.parse::<SocketAddr>() {
            return self.get_mut_by_address(&address);
        } else {
            return self.get_mut_by_name(identifier);
        }
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Connection> {
        self.connections.values_mut()
    }
}

impl Drop for ConnectionMap {
    fn drop(&mut self) {
        for connection in self.connections.drain() {
            drop(connection.1);
        }
    }
}