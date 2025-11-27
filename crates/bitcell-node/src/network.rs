///! Network manager with TCP-based P2P communication

use crate::{Result, MetricsRegistry};
use bitcell_consensus::{Block, Transaction};
use bitcell_crypto::PublicKey;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Handshake with peer ID
    Handshake { peer_id: PublicKey },
    /// Ping to keep connection alive
    Ping,
    /// Pong response
    Pong,
    /// Broadcast a new block
    Block(Block),
    /// Broadcast a transaction
    Transaction(Transaction),
    /// Request peers list
    GetPeers,
    /// Response with peers list
    Peers(Vec<String>), // List of "ip:port" addresses
}

/// Peer connection info
struct PeerConnection {
    peer_id: PublicKey,
    address: String,
    writer: Arc<RwLock<Option<tokio::io::WriteHalf<TcpStream>>>>,
}

/// Network manager with real TCP networking
#[derive(Clone)]
pub struct NetworkManager {
    /// Local peer ID (node's public key)
    local_peer: PublicKey,
    
    /// Local listening address
    local_addr: Arc<RwLock<Option<String>>>,
    
    /// Connected peers
    peers: Arc<RwLock<HashMap<PublicKey, PeerConnection>>>,
    
    /// Known peer addresses for discovery
    known_addresses: Arc<RwLock<HashSet<String>>>,
    
    /// Metrics registry
    metrics: MetricsRegistry,
    
    /// Block broadcast channel
    block_tx: Arc<RwLock<Option<mpsc::Sender<Block>>>>,
    
    /// Transaction broadcast channel
    tx_tx: Arc<RwLock<Option<mpsc::Sender<Transaction>>>>,
    
    /// DHT manager
    dht: Arc<RwLock<Option<crate::dht::DhtManager>>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(local_peer: PublicKey, metrics: MetricsRegistry) -> Self {
        Self {
            local_peer,
            local_addr: Arc::new(RwLock::new(None)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            known_addresses: Arc::new(RwLock::new(HashSet::new())),
            metrics,
            block_tx: Arc::new(RwLock::new(None)),
            tx_tx: Arc::new(RwLock::new(None)),
            dht: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Enable DHT
    pub fn enable_dht(&self, secret_key: &bitcell_crypto::SecretKey, bootstrap: Vec<String>) -> Result<()> {
        let dht_manager = crate::dht::DhtManager::new(secret_key, bootstrap)?;
        let mut dht = self.dht.write();
        *dht = Some(dht_manager);
        println!("DHT enabled");
        Ok(())
    }
    
    /// Start the network listener
    pub async fn start(&self, port: u16, bootstrap_nodes: Vec<String>) -> Result<()> {
        let addr = format!("0.0.0.0:{}", port);
        
        // Update local address
        {
            let mut local_addr = self.local_addr.write();
            *local_addr = Some(format!("127.0.0.1:{}", port));
        }
        
        // Bind to the port
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;
        
        println!("Network listening on {}", addr);
        
        // Spawn listener task
        let network = self.clone();
        tokio::spawn(async move {
            network.accept_connections(listener).await;
        });
        
        // Start DHT discovery if enabled
        let dht_clone = self.dht.clone();
        let network_clone = self.clone();
        let bootstrap_nodes_clone = bootstrap_nodes.clone();
        
        tokio::spawn(async move {
            // Wait a bit for listener to start
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            
            let mut dht_manager = {
                let mut guard = dht_clone.write();
                guard.take()
            };
            
            if let Some(mut dht) = dht_manager {
                println!("Starting DHT discovery...");
                
                // 1. Connect to explicit bootstrap nodes from config
                // This is necessary because DhtManager might reject addresses without Peer IDs
                if !bootstrap_nodes_clone.is_empty() {
                    println!("Connecting to {} bootstrap nodes...", bootstrap_nodes_clone.len());
                    for addr_str in bootstrap_nodes_clone {
                        // Extract IP and port from multiaddr string /ip4/x.x.x.x/tcp/yyyy
                        // Also handle /p2p/Qm... suffix if present
                        if let Some(start) = addr_str.find("/ip4/") {
                            if let Some(tcp_start) = addr_str.find("/tcp/") {
                                let ip = &addr_str[start+5..tcp_start];
                                let rest = &addr_str[tcp_start+5..];
                                
                                // Check if there's a /p2p/ or /ipfs/ suffix
                                let port = if let Some(p2p_start) = rest.find("/p2p/") {
                                    &rest[..p2p_start]
                                } else if let Some(ipfs_start) = rest.find("/ipfs/") {
                                    &rest[..ipfs_start]
                                } else {
                                    rest
                                };
                                
                                let connect_addr = format!("{}:{}", ip, port);
                                println!("Connecting to bootstrap node: {}", connect_addr);
                                let _ = network_clone.connect_to_peer(&connect_addr).await;
                            }
                        }
                    }
                }

                if let Ok(peers) = dht.start_discovery().await {
                    println!("DHT discovery found {} peers", peers.len());
                    for peer in peers {
                        for addr in peer.addresses {
                            // Convert multiaddr to string address if possible
                            // For now, we assume TCP/IP addresses
                            // This is a simplification - in a real implementation we'd handle Multiaddr properly
                            let addr_str = addr.to_string();
                            // Extract IP and port from multiaddr string /ip4/x.x.x.x/tcp/yyyy
                            if let Some(start) = addr_str.find("/ip4/") {
                                if let Some(tcp_start) = addr_str.find("/tcp/") {
                                    let ip = &addr_str[start+5..tcp_start];
                                    let rest = &addr_str[tcp_start+5..];
                                    
                                    // Check if there's a /p2p/ or /ipfs/ suffix
                                    let port = if let Some(p2p_start) = rest.find("/p2p/") {
                                        &rest[..p2p_start]
                                    } else if let Some(ipfs_start) = rest.find("/ipfs/") {
                                        &rest[..ipfs_start]
                                    } else {
                                        rest
                                    };
                                    
                                    let connect_addr = format!("{}:{}", ip, port);
                                    println!("DHT discovered peer: {}", connect_addr);
                                    let _ = network_clone.connect_to_peer(&connect_addr).await;
                                }
                            }
                        }
                    }
                }
                
                // Put it back
                let mut guard = dht_clone.write();
                *guard = Some(dht);
            }
        });
        
        // Spawn peer discovery task
        let network = self.clone();
        tokio::spawn(async move {
            network.peer_discovery_loop().await;
        });
        
        self.metrics.set_peer_count(self.peer_count());
        
        Ok(())
    }
    
    /// Accept incoming connections
    async fn accept_connections(&self, listener: TcpListener) {
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    println!("Accepted connection from {}", addr);
                    let network = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = network.handle_connection(socket).await {
                            eprintln!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    }
    
    /// Handle a peer connection
    async fn handle_connection(&self, mut socket: TcpStream) -> Result<()> {
        println!("Accepted connection");
        
        // Send handshake
        self.send_message(&mut socket, &NetworkMessage::Handshake { peer_id: self.local_peer }).await?;
        println!("Sent handshake to incoming peer");
        
        // Read handshake response
        let msg = self.receive_message(&mut socket).await?;
        println!("Received handshake response");
        
        let peer_id = match msg {
            NetworkMessage::Handshake { peer_id } => peer_id,
            _ => return Err("Expected handshake".into()),
        };
        
        println!("Handshake complete with peer: {:?}", peer_id);
        
        // Split socket for concurrent read/write
        let (reader, writer) = tokio::io::split(socket);
        
        // Store peer connection
        {
            let mut peers = self.peers.write();
            peers.insert(peer_id, PeerConnection {
                peer_id,
                address: "unknown".to_string(),
                writer: Arc::new(RwLock::new(Some(writer))),
            });
            self.metrics.set_peer_count(peers.len());
        }
        
        // Handle incoming messages
        self.handle_messages(reader, peer_id).await?;
        
        // Remove peer on disconnect
        {
            let mut peers = self.peers.write();
            peers.remove(&peer_id);
            self.metrics.set_peer_count(peers.len());
        }
        
        Ok(())
    }
    
    /// Handle incoming messages from a peer
    async fn handle_messages(&self, mut reader: tokio::io::ReadHalf<TcpStream>, peer_id: PublicKey) -> Result<()> {
        loop {
            match self.receive_message_from_reader(&mut reader).await {
                Ok(msg) => {
                    match msg {
                        NetworkMessage::Ping => {
                            // Respond with pong
                            self.send_to_peer(&peer_id, &NetworkMessage::Pong).await?;
                        }
                        NetworkMessage::Pong => {
                            
                        }
                        NetworkMessage::Block(block) => {
                            println!("Received block {} from peer", block.header.height);
                            self.handle_incoming_block(block).await?;
                        }
                        NetworkMessage::Transaction(tx) => {
                            println!("Received transaction from peer");
                            self.handle_incoming_transaction(tx).await?;
                        }
                        NetworkMessage::GetPeers => {
                            let addresses: Vec<String> = {
                                let known = self.known_addresses.read();
                                known.iter().cloned().collect()
                            };
                            self.send_to_peer(&peer_id, &NetworkMessage::Peers(addresses)).await?;
                        }
                        NetworkMessage::Peers(addresses) => {
                            // Add new peer addresses
                            let mut known = self.known_addresses.write();
                            for addr in addresses {
                                known.insert(addr);
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    println!("Peer {:?} disconnected: {}", peer_id, e);
                    break;
                }
            }
        }
        Ok(())
    }
    
    /// Send a message to a peer
    async fn send_to_peer(&self, peer_id: &PublicKey, msg: &NetworkMessage) -> Result<()> {
        // Obtain the writer for the target peer without holding the lock across await
        let writer_arc_opt = {
            let peers = self.peers.read();
            peers.get(peer_id).map(|peer| peer.writer.clone())
        };
        if let Some(writer_arc) = writer_arc_opt {
            // Take the writer out of the lock
            let mut writer_opt = {
                let mut guard = writer_arc.write();
                guard.take()
            };
            if let Some(mut writer) = writer_opt {
                // Serialize the message
                let data = bincode::serialize(msg)
                    .map_err(|e| format!("Serialization error: {}", e))?;
                let len = (data.len() as u32).to_be_bytes();
                // Send length prefix and data
                writer.write_all(&len).await
                    .map_err(|e| format!("Write error: {}", e))?;
                writer.write_all(&data).await
                    .map_err(|e| format!("Write error: {}", e))?;
                writer.flush().await
                    .map_err(|e| format!("Flush error: {}", e))?;
                // Return writer to the lock
                let mut guard = writer_arc.write();
                *guard = Some(writer);
                // Update metrics
                self.metrics.add_bytes_sent(data.len() as u64);
            }
        }
        Ok(())
    }
    
    /// Send a message over a socket
    async fn send_message(&self, socket: &mut TcpStream, msg: &NetworkMessage) -> Result<()> {
        let data = bincode::serialize(msg)
            .map_err(|e| format!("Serialization error: {}", e))?;
        let len = data.len() as u32;
        
        socket.write_all(&len.to_be_bytes()).await
            .map_err(|e| format!("Write error: {}", e))?;
        socket.write_all(&data).await
            .map_err(|e| format!("Write error: {}", e))?;
        socket.flush().await
            .map_err(|e| format!("Flush error: {}", e))?;
        
        Ok(())
    }
    
    /// Receive a message from a socket
    async fn receive_message(&self, socket: &mut TcpStream) -> Result<NetworkMessage> {
        let mut len_bytes = [0u8; 4];
        socket.read_exact(&mut len_bytes).await
            .map_err(|e| format!("Read error: {}", e))?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        if len > 10_000_000 { // 10MB safety limit
            return Err("Message too large".into());
        }
        
        let mut data = vec![0u8; len];
        socket.read_exact(&mut data).await
            .map_err(|e| format!("Read error: {}", e))?;
        
        let msg = bincode::deserialize(&data)
            .map_err(|e| format!("Deserialization error: {}", e))?;
        
        Ok(msg)
    }
    
    /// Receive a message from a reader
    async fn receive_message_from_reader(&self, reader: &mut tokio::io::ReadHalf<TcpStream>) -> Result<NetworkMessage> {
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes).await
            .map_err(|e| format!("Read error: {}", e))?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        if len > 10_000_000 { // 10MB safety limit
            return Err("Message too large".into());
        }
        
        let mut data = vec![0u8; len];
        reader.read_exact(&mut data).await
            .map_err(|e| format!("Read error: {}", e))?;
        
        let msg = bincode::deserialize(&data)
            .map_err(|e| format!("Deserialization error: {}", e))?;
        
        Ok(msg)
    }
    
    /// Connect to a peer
    pub async fn connect_to_peer(&self, address: &str) -> Result<()> {
        // Don't connect to ourselves
        if let Some(ref local) = *self.local_addr.read() {
            if address == local {
                return Ok(());
            }
        }
        
        // Check if already connected
        {
            let peers = self.peers.read();
            for peer in peers.values() {
                if peer.address == address {
                    return Ok(());
                }
            }
        }
        
        // Only print if we're actually attempting a new connection
        println!("Connecting to peer at {}", address);
        
        match TcpStream::connect(address).await {
            Ok(mut socket) => {
                println!("Connected to {}, sending handshake", address);
                // Send handshake
                self.send_message(&mut socket, &NetworkMessage::Handshake {
                    peer_id: self.local_peer,
                }).await?;
                println!("Sent handshake to {}", address);
                
                // Receive handshake
                let msg = self.receive_message(&mut socket).await?;
                println!("Received handshake response from {}", address);
                
                let peer_id = match msg {
                    NetworkMessage::Handshake { peer_id } => peer_id,
                    _ => return Err("Expected handshake".into()),
                };
                
                println!("Connected to peer: {:?}", peer_id);
                
                // Split socket
                let (reader, writer) = tokio::io::split(socket);
                
                // Store peer
                {
                    let mut peers = self.peers.write();
                    peers.insert(peer_id, PeerConnection {
                        peer_id,
                        address: address.to_string(),
                        writer: Arc::new(RwLock::new(Some(writer))),
                    });
                    self.metrics.set_peer_count(peers.len());
                    self.metrics.set_dht_peer_count(peers.len()); // Show TCP peers as DHT peers
                }
                
                // Handle messages from this peer
                let network = self.clone();
                tokio::spawn(async move {
                    let _ = network.handle_messages(reader, peer_id).await;
                });
                
                // Add to known addresses
                {
                    let mut known = self.known_addresses.write();
                    known.insert(address.to_string());
                }
                
                Ok(())
            }
            Err(e) => {
                Err(format!("Failed to connect to {}: {}", address, e).into())
            }
        }
    }
    
    /// Peer discovery loop
    async fn peer_discovery_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Get list of known addresses and filter out ones we're already connected to
            let addresses_to_try: Vec<String> = {
                let known = self.known_addresses.read();
                let peers = self.peers.read();
                
                // Collect all currently connected addresses
                let connected_addrs: std::collections::HashSet<String> = peers
                    .values()
                    .map(|p| p.address.clone())
                    .collect();
                
                // Only try addresses we're not connected to
                known
                    .iter()
                    .filter(|addr| !connected_addrs.contains(*addr))
                    .cloned()
                    .collect()
            };
            
            // Try to connect to new addresses only
            for addr in addresses_to_try {
                let _ = self.connect_to_peer(&addr).await;
            }
            
            // Request more peers from connected peers
            let peer_ids: Vec<PublicKey> = {
                let peers = self.peers.read();
                peers.keys().copied().collect()            };
            
            for peer_id in peer_ids {
                let _ = self.send_to_peer(&peer_id, &NetworkMessage::GetPeers).await;
            }
        }
    }
    
    /// Connect to a peer by PublicKey (legacy compatibility)
    pub fn connect_peer(&self, peer_id: PublicKey) -> Result<()> {
        // This is now handled by connect_to_peer with actual addresses
        println!("Legacy connect_peer called for: {:?}", peer_id);
        Ok(())
    }
    
    /// Disconnect from a peer
    pub fn disconnect_peer(&self, peer_id: &PublicKey) -> Result<()> {
        let mut peers = self.peers.write();
        peers.remove(peer_id);
        self.metrics.set_peer_count(peers.len());
        println!("Disconnected from peer: {:?}", peer_id);
        Ok(())
    }
    
    /// Broadcast a block to all connected peers
    pub async fn broadcast_block(&self, block: &Block) -> Result<()> {
        let peer_ids: Vec<PublicKey> = {
            let peers = self.peers.read();
            println!("Broadcasting block {} to {} peers", block.header.height, peers.len());
            peers.keys().copied().collect()
        };
        
        let msg = NetworkMessage::Block(block.clone());
        let data = bincode::serialize(&msg).unwrap_or_default();
        let block_size = data.len() as u64;
        
        for peer_id in &peer_ids {
            let _ = self.send_to_peer(peer_id, &msg).await;
        }
        
        self.metrics.add_bytes_sent(block_size * peer_ids.len() as u64);
        Ok(())
    }
    
    /// Broadcast a transaction to all connected peers
    pub async fn broadcast_transaction(&self, tx: &Transaction) -> Result<()> {
        let peer_ids: Vec<PublicKey> = {
            let peers = self.peers.read();
            println!("Broadcasting transaction to {} peers", peers.len());
            peers.keys().copied().collect()
        };
        
        let msg = NetworkMessage::Transaction(tx.clone());
        let data = bincode::serialize(&msg).unwrap_or_default();
        let tx_size = data.len() as u64;
        
        for peer_id in &peer_ids {
            let _ = self.send_to_peer(peer_id, &msg).await;
        }
        
        self.metrics.add_bytes_sent(tx_size * peer_ids.len() as u64);
        Ok(())
    }
    
    /// Get number of connected peers
    pub fn peer_count(&self) -> usize {
        self.peers.read().len()
    }
    
    /// Get list of connected peers
    pub fn connected_peers(&self) -> Vec<PublicKey> {
        self.peers.read().keys().copied().collect()
    }
    
    /// Handle incoming block from network
    pub async fn handle_incoming_block(&self, block: Block) -> Result<()> {
        let block_size = bincode::serialize(&block).unwrap_or_default().len() as u64;
        self.metrics.add_bytes_received(block_size);
        
        // Forward to block processing channel
        let tx_opt = {
            let guard = self.block_tx.read();
            guard.as_ref().cloned()
        };
        if let Some(tx) = tx_opt {
            let _ = tx.send(block).await;
        }
        
        Ok(())
    }
    
    /// Handle incoming transaction from network
    pub async fn handle_incoming_transaction(&self, tx: Transaction) -> Result<()> {
        let tx_size = bincode::serialize(&tx).unwrap_or_default().len() as u64;
        self.metrics.add_bytes_received(tx_size);
        
        // Forward to transaction processing channel
        let sender_opt = {
            let guard = self.tx_tx.read();
            guard.as_ref().cloned()
        };
        if let Some(sender) = sender_opt {
            let _ = sender.send(tx).await;
        }
        
        Ok(())
    }
    
    /// Set block broadcast channel
    pub fn set_block_channel(&self, tx: mpsc::Sender<Block>) {
        let mut block_tx = self.block_tx.write();
        *block_tx = Some(tx);
    }
    
    /// Set transaction broadcast channel
    pub fn set_tx_channel(&self, tx: mpsc::Sender<Transaction>) {
        let mut tx_tx = self.tx_tx.write();
        *tx_tx = Some(tx);
    }
    
    /// Add a bootstrap peer address (for initial connection)
    pub fn add_bootstrap_peer(&self, address: String) {
        let mut known = self.known_addresses.write();
        known.insert(address);
    }
}

/// Peer discovery helper - connect to bootstrap peers
pub async fn discover_peers(
    network: Arc<NetworkManager>,
    bootstrap_addresses: Vec<String>,
) -> Result<()> {
    println!("Starting peer discovery with {} bootstrap addresses...", bootstrap_addresses.len());
    
    for addr in bootstrap_addresses {
        network.add_bootstrap_peer(addr.clone());
        if let Err(e) = network.connect_to_peer(&addr).await {
            eprintln!("Failed to connect to bootstrap peer {}: {}", addr, e);
        }
    }
    
    println!("Peer discovery complete: {} peers connected", network.peer_count());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcell_crypto::SecretKey;
    
    #[test]
    fn test_network_creation() {
        let sk = SecretKey::generate();
        let pk = sk.public_key();
        let metrics = MetricsRegistry::new();
        
        let network = NetworkManager::new(pk, metrics);
        assert_eq!(network.peer_count(), 0);
    }
}
