use crate::protocols::outlet::{OutletRequest, OutletResponse, OutletState};
use crate::simulators::errors::SimulatorErrors;
use crate::smart_devices::Watt;
use std::io;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Configuration for the outlet simulator
#[derive(Debug, Clone)]
pub struct OutletSimulatorConfig {
    /// TCP address to bind to (e.g., "127.0.0.1:8001")
    pub tcp_addr: String,
    /// Power consumption when outlet is ON (in watts)
    pub power_watts: Watt,
}

impl OutletSimulatorConfig {
    pub fn new(tcp_addr: impl Into<String>, power_watts: Watt) -> Self {
        Self {
            tcp_addr: tcp_addr.into(),
            power_watts,
        }
    }
}

/// Internal simulated outlet device with state
struct SimulatedOutletCtx {
    state: OutletState,
    power_watts: Watt,
}

impl SimulatedOutletCtx {
    fn new(power_watts: Watt) -> Self {
        Self {
            state: OutletState::Off,
            power_watts,
        }
    }

    /// Process a request and return the appropriate response
    fn handle_request(&mut self, request: OutletRequest) -> OutletResponse {
        match request {
            OutletRequest::GetState => OutletResponse::State(self.state),
            OutletRequest::GetPower => {
                let power = match self.state {
                    OutletState::On => self.power_watts,
                    OutletState::Off => 0,
                };
                OutletResponse::Power(power as u32)
            }
            OutletRequest::TurnOn => {
                self.state = OutletState::On;
                OutletResponse::Ok
            }
            OutletRequest::TurnOff => {
                self.state = OutletState::Off;
                OutletResponse::Ok
            }
            OutletRequest::Switch => {
                self.state = match self.state {
                    OutletState::On => OutletState::Off,
                    OutletState::Off => OutletState::On,
                };
                OutletResponse::Ok
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimulatorStatus {
    Starting,
    Running,
    Stopped,
    Error(String),
}

/// Handle to a running outlet simulator
/// When dropped, the simulator is automatically stopped
pub struct OutletSimulator {
    listener_thread: Option<JoinHandle<()>>,
    shutdown_flag: Arc<AtomicBool>,
    addr: String,
    status: Arc<Mutex<SimulatorStatus>>,
}

impl OutletSimulator {
    /// Spawn a new outlet simulator
    ///
    /// Returns a handle to the simulator that will automatically
    /// clean up when dropped.
    pub fn spawn(config: OutletSimulatorConfig) -> Result<Self, SimulatorErrors> {
        let listener = TcpListener::bind(&config.tcp_addr)
            .map_err(|e| SimulatorErrors::Outlet(e.to_string()))?;
        let actual_addr = listener
            .local_addr()
            .map_err(|e| SimulatorErrors::Outlet(e.to_string()))?
            .to_string();
        let status = Arc::new(Mutex::new(SimulatorStatus::Starting));
        let status_clone = status.clone();

        // Make listener non-blocking for graceful shutdown
        listener
            .set_nonblocking(true)
            .map_err(|e| SimulatorErrors::Outlet(e.to_string()))?;

        let outlet = Arc::new(Mutex::new(SimulatedOutletCtx::new(config.power_watts)));
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let shutdown_flag_cloned = shutdown_flag.clone();
        let listener_thread = thread::spawn(move || {
            Self::run_server(listener, outlet, status_clone, shutdown_flag_cloned);
        });

        Ok(Self {
            listener_thread: Some(listener_thread),
            shutdown_flag,
            addr: actual_addr,
            status,
        })
    }

    /// Get the actual address the simulator is bound to
    pub fn address(&self) -> &str {
        &self.addr
    }

    #[allow(dead_code)]
    pub fn status(&self) -> SimulatorStatus {
        if let Ok(status_guard) = self.status.lock() {
            return status_guard.clone();
        }
        SimulatorStatus::Error("Failed to acquire status lock".to_string())
    }

    pub fn wait_until_running(&self, max_wait: std::time::Duration) -> Result<(), SimulatorStatus> {
        let start = std::time::Instant::now();
        loop {
            if self.shutdown_flag.load(Ordering::Relaxed) {
                return Err(SimulatorStatus::Stopped);
            }
            if let Ok(status_guard) = self.status.lock()
                && let SimulatorStatus::Running = *status_guard
            {
                return Ok(());
            }
            if start.elapsed() >= max_wait {
                return Err(SimulatorStatus::Error(
                    "Timeout waiting for simulator to start".to_string(),
                ));
            }
            thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    /// Run the TCP server loop
    fn run_server(
        listener: TcpListener,
        outlet: Arc<Mutex<SimulatedOutletCtx>>,
        status: Arc<Mutex<SimulatorStatus>>,
        shutdown: Arc<AtomicBool>,
    ) {
        if let Ok(mut status_guard) = status.lock() {
            *status_guard = SimulatorStatus::Running;
        }
        loop {
            if shutdown.load(Ordering::SeqCst) {
                break;
            }
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let outlet = outlet.clone();
                    let shutdown = shutdown.clone();
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, outlet, shutdown) {
                            eprintln!("Client handler error: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No connection available, sleep briefly
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    break;
                }
            }
        }
        if let Ok(mut status_guard) = status.lock() {
            *status_guard = SimulatorStatus::Stopped;
        }
    }

    /// Handle a single client connection
    fn handle_client(
        mut stream: TcpStream,
        outlet: Arc<Mutex<SimulatedOutletCtx>>,
        shutdown: Arc<AtomicBool>,
    ) -> io::Result<()> {
        // Set stream to blocking mode (it may inherit non-blocking from listener on some platforms)
        stream.set_nonblocking(false)?;

        loop {
            if shutdown.load(Ordering::SeqCst) {
                stream.shutdown(Shutdown::Both)?;
                break;
            }
            // Receive request
            let request = match OutletRequest::receive(&mut stream) {
                Ok(req) => req,
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    println!("Client disconnected");
                    break; // Client disconnected
                }
                Err(e) => return Err(e),
            };

            // Handle request
            let response = {
                let mut outlet = outlet.lock().unwrap();
                outlet.handle_request(request)
            };

            // Send response
            response.send(&mut stream)?;
        }
        println!("Stop client handler");
        Ok(())
    }
}

impl Drop for OutletSimulator {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown_flag.store(true, Ordering::Relaxed);
        // Wait for listener thread to finish
        if let Some(handle) = self.listener_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_devices::OutletDevice;
    use crate::smart_devices::outlet_remote::OutletRemote;
    use crate::smart_devices::types::OutletState as DeviceOutletState;
    use crate::traits::Information;
    use std::time::Duration;

    #[test]
    fn test_simulator_spawn_and_connect() {
        // Use port 0 to get a random available port
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 100);
        let simulator = match OutletSimulator::spawn(config) {
            Ok(sim) => sim,
            Err(e) => {
                panic!("Failed to spawn simulator: {}", e);
            }
        };
        simulator
            .wait_until_running(Duration::from_millis(500))
            .unwrap();

        thread::sleep(Duration::from_millis(100));

        let addr = simulator.address().to_string();
        let outlet = OutletRemote::new("Test Outlet".to_string(), addr);

        assert!(outlet.is_ok());
    }

    #[test]
    fn test_simulator_functionality() {
        let config = OutletSimulatorConfig::new("127.0.0.1:9123", 150);
        let simulator = match OutletSimulator::spawn(config) {
            Ok(sim) => sim,
            Err(e) => {
                panic!("Failed to spawn simulator: {}", e);
            }
        };
        simulator
            .wait_until_running(Duration::from_millis(100))
            .unwrap();

        let addr = simulator.address().to_string();
        let outlet = OutletRemote::new("Test Outlet".to_string(), addr);
        let mut outlet = match outlet {
            Ok(o) => o,
            Err(e) => {
                panic!("Failed to create remote outlet: {}", e);
            }
        };

        // Initial state should be Off
        assert_eq!(outlet.state().unwrap(), DeviceOutletState::Off);
        assert_eq!(outlet.power_usage().unwrap(), 0);

        // Turn on
        outlet.turn_on().unwrap();
        assert_eq!(outlet.state().unwrap(), DeviceOutletState::On);
        assert_eq!(outlet.power_usage().unwrap(), 150);

        // Get info
        let info = outlet.info();
        assert_eq!(
            info,
            "Remote Smart Outlet: Test Outlet - Current State: On, Power Usage: 150 Watt"
        );

        // Turn off
        outlet.turn_off().unwrap();
        assert_eq!(outlet.state().unwrap(), DeviceOutletState::Off);
        assert_eq!(outlet.power_usage().unwrap(), 0);
    }

    #[test]
    fn test_multiple_clients() {
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 200);
        let simulator = OutletSimulator::spawn(config).unwrap();

        simulator
            .wait_until_running(Duration::from_millis(100))
            .unwrap();

        // Connect multiple clients
        let mut client1 =
            OutletRemote::new("Client1".to_string(), simulator.address().to_string()).unwrap();

        let client2 =
            OutletRemote::new("Client2".to_string(), simulator.address().to_string()).unwrap();

        // Client 1 turns on
        client1.turn_on().unwrap();

        // Client 2 should see the same state
        assert_eq!(client2.state().unwrap(), DeviceOutletState::On);
        assert_eq!(client2.power_usage().unwrap(), 200);
    }

    #[test]
    fn test_simulator_cleanup() {
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 100);
        let addr = {
            let simulator = OutletSimulator::spawn(config).unwrap();
            simulator
                .wait_until_running(Duration::from_millis(100))
                .unwrap();
            simulator.address().to_string()
        }; // Simulator dropped here

        // Should not be able to connect after drop
        let _ = OutletRemote::new("Test".to_string(), addr);
        // Connection might fail or succeed depending on timing
        // This test mainly ensures no panic on drop
    }
}
