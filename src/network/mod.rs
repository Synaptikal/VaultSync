use anyhow::Result;
use chrono::{DateTime, Utc};
pub use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NetworkService {
    devices: Arc<RwLock<Vec<Device>>>,
    is_discovering: bool,
    daemon: Option<ServiceDaemon>,
    node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub address: IpAddr,
    pub port: u16,
    pub service_type: String,
    pub last_seen: DateTime<Utc>,
    pub status: DeviceStatus,
    pub node_id: Option<String>,
}

impl NetworkService {
    pub fn new() -> Result<Self> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| anyhow::anyhow!("Failed to create mDNS daemon: {}", e))?;

        Ok(Self {
            devices: Arc::new(RwLock::new(Vec::new())),
            is_discovering: false,
            daemon: Some(daemon),
            node_id: String::new(), // Will be set in start_discovery
        })
    }

    pub async fn start_discovery(&mut self, node_id: &str, port: u16) -> Result<()> {
        let my_ip = local_ip_address::local_ip()
            .map_err(|e| anyhow::anyhow!("Failed to get local IP: {}", e))?;

        tracing::info!(
            "Starting mDNS discovery for VaultSync node: {} on {}:{}",
            node_id,
            my_ip,
            port
        );
        self.is_discovering = true;
        self.node_id = node_id.to_string();

        // Register our service so other devices can find us
        self.register_service(node_id, port)?;

        // Start listening for other VaultSync services
        self.listen_for_services(my_ip, port).await?;

        // Start heartbeat check task
        let devices = self.devices.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                Self::check_stale_devices(&devices).await;
            }
        });

        Ok(())
    }

    /// Check and mark devices as offline if not seen recently (TASK-116)
    async fn check_stale_devices(devices: &Arc<RwLock<Vec<Device>>>) {
        let mut devices_guard = devices.write().await;
        let now = Utc::now();
        let stale_threshold = chrono::Duration::seconds(90); // 90 seconds without heartbeat = offline

        for device in devices_guard.iter_mut() {
            if device.status == DeviceStatus::Online {
                let age = now - device.last_seen;
                if age > stale_threshold {
                    tracing::warn!(
                        "Device {} marked offline (last seen {} ago)",
                        device.name,
                        age
                    );
                    device.status = DeviceStatus::Offline;
                }
            }
        }
    }

    /// Get only online devices (TASK-115)
    pub async fn get_online_devices(&self) -> Vec<Device> {
        self.devices
            .read()
            .await
            .iter()
            .filter(|d| d.status == DeviceStatus::Online)
            .cloned()
            .collect()
    }

    /// Get all devices regardless of status
    pub async fn get_connected_devices(&self) -> Vec<Device> {
        self.devices.read().await.clone()
    }

    async fn _simulate_device_discovery(&self) {
        // Removed simulation
    }

    pub fn register_service(&mut self, node_id: &str, port: u16) -> Result<()> {
        if let Some(ref daemon) = self.daemon {
            // Get local IP
            let my_ip = local_ip_address::local_ip()
                .map_err(|e| anyhow::anyhow!("Failed to get local IP: {}", e))?;

            tracing::info!("Registering mDNS service on {}", my_ip);

            // Create service information for VaultSync
            // Service type: _vaultsync._tcp.local.
            // Instance name: VaultSync-{node_id}
            let instance_name = format!("VaultSync-{}", node_id);
            let hostname = format!("{}.local.", node_id);

            let service_info = ServiceInfo::new(
                "_vaultsync._tcp.local.",
                &instance_name,
                &hostname,
                &my_ip.to_string(),
                port,
                std::collections::HashMap::from([
                    ("version".to_string(), "1.0.0".to_string()),
                    ("node_id".to_string(), node_id.to_string()),
                ]),
            )
            .map_err(|e| anyhow::anyhow!("Failed to create service info: {}", e))?;

            // Register the service
            daemon
                .register(service_info)
                .map_err(|e| anyhow::anyhow!("Failed to register service: {}", e))?;

            tracing::info!(
                "Registered service: {} on {}:{}",
                instance_name,
                my_ip,
                port
            );
        }

        Ok(())
    }

    async fn listen_for_services(&self, local_ip: IpAddr, local_port: u16) -> Result<()> {
        if let Some(ref daemon) = self.daemon {
            // Start browsing for VaultSync services
            let receiver = daemon
                .browse("_vaultsync._tcp.local.")
                .map_err(|e| anyhow::anyhow!("Failed to start browsing: {}", e))?;

            // Spawn a task to handle service discovery events
            let devices = self.devices.clone();
            tokio::spawn(async move {
                while let Ok(event) = receiver.recv() {
                    match event {
                        ServiceEvent::ServiceFound(_ty, fullname) => {
                            tracing::debug!("Found service: {}", fullname);
                        }
                        ServiceEvent::ServiceResolved(info) => {
                            tracing::info!(
                                "Resolved service: {} at {:?}",
                                info.get_fullname(),
                                info.get_addresses()
                            );

                            // Check if this is self
                            let port = info.get_port();
                            let is_self = port == local_port
                                && info.get_addresses().iter().any(|ip| *ip == local_ip);

                            if is_self {
                                tracing::debug!("Ignoring self-discovered service");
                                continue;
                            }

                            // Add the resolved service to our device list
                            let mut devices_guard = devices.write().await;

                            // Check if already exists to avoid duplicates
                            let hostname = info.get_hostname();
                            let remote_node_id =
                                info.get_properties().get("node_id").map(|s| s.to_string());

                            if !devices_guard.iter().any(|d| d.name == hostname) {
                                for addr in info.get_addresses().iter() {
                                    tracing::info!("Adding peer device: {} ({})", hostname, addr);
                                    devices_guard.push(Device {
                                        name: hostname.to_string(),
                                        address: *addr,
                                        port,
                                        service_type: info.get_type().to_string(),
                                        last_seen: Utc::now(),
                                        status: DeviceStatus::Online,
                                        node_id: remote_node_id.clone(),
                                    });
                                    // Only add first valid address for now
                                    break;
                                }
                            } else {
                                // Update last_seen for existing device
                                if let Some(device) =
                                    devices_guard.iter_mut().find(|d| d.name == hostname)
                                {
                                    device.last_seen = Utc::now();
                                    device.status = DeviceStatus::Online;
                                }
                            }
                        }
                        ServiceEvent::ServiceRemoved(_ty, fullname) => {
                            tracing::info!("Removed service: {}", fullname);
                            // Ideally remove from devices list
                            // Currently just logging as we can't map fullname easily without better tracking
                        }
                        _ => {}
                    }
                }
            });
        }

        Ok(())
    }

    /// Manually add a device by IP and port (TASK-118: Manual pairing fallback)
    pub async fn manual_add_device(
        &self,
        name: String,
        address: IpAddr,
        port: u16,
        node_id: Option<String>,
    ) -> Result<()> {
        let mut devices_guard = self.devices.write().await;

        // Check if already exists
        if devices_guard
            .iter()
            .any(|d| d.address == address && d.port == port)
        {
            tracing::info!("Device {}:{} already exists", address, port);
            return Ok(());
        }

        devices_guard.push(Device {
            name,
            address,
            port,
            service_type: "_vaultsync._tcp.local.".to_string(),
            last_seen: Utc::now(),
            status: DeviceStatus::Online,
            node_id,
        });

        tracing::info!("Manually added device at {}:{}", address, port);
        Ok(())
    }

    /// Remove a device by address
    pub async fn remove_device(&self, address: IpAddr, port: u16) -> bool {
        let mut devices_guard = self.devices.write().await;
        let initial_len = devices_guard.len();
        devices_guard.retain(|d| !(d.address == address && d.port == port));
        devices_guard.len() < initial_len
    }
}
