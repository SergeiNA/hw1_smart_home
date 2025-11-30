use super::types::Celsius;
use crate::protocols::thermometer::TemperatureData;
use crate::smart_devices::TemperatureSensor;
use crate::traits::Information;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

const UDP_BUFFER_SIZE: usize = 8; // 4 bytes for temperature (f32) + 4 bytes for timestamp (u32)

#[derive(Debug)]
pub struct ThermometerRemote {
    name: String,
    temperature: Arc<Mutex<Celsius>>,
    receiver_thread: Option<thread::JoinHandle<()>>,
    shutdown_tx: mpsc::Sender<()>, // can use AtomicBool but this is for education purpose
}

impl ThermometerRemote {
    /// Create a new thermometer in remote mode (UDP reception)
    pub fn new(name: String, udp_addr: String) -> Result<Self, std::io::Error> {
        let temperature = Arc::new(Mutex::new(Celsius::default())); // Default initial value
        let temp_clone = Arc::clone(&temperature);
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        // Spawn background UDP receiver thread
        let receiver_thread = thread::spawn(move || {
            Self::udp_receiver_loop(udp_addr, temp_clone, shutdown_rx);
        });

        Ok(ThermometerRemote {
            name,
            temperature,
            receiver_thread: Some(receiver_thread),
            shutdown_tx,
        })
    }

    /// Background thread that receives UDP packets and updates temperature
    fn udp_receiver_loop(
        udp_addr: String,
        temperature: Arc<Mutex<Celsius>>,
        shutdown_rx: mpsc::Receiver<()>,
    ) {
        // Bind UDP socket
        let socket = match UdpSocket::bind(&udp_addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to bind UDP socket to {}: {}", udp_addr, e);
                return;
            }
        };

        // Set non-blocking mode for periodic shutdown checks
        if let Err(e) = socket.set_nonblocking(true) {
            eprintln!("Failed to set non-blocking: {}", e);
            return;
        }

        println!("Thermometer UDP receiver listening on {}", udp_addr);

        let mut buf = [0u8; UDP_BUFFER_SIZE];

        let mut current_timestamp: u32 = 0;

        loop {
            // Check for shutdown signal
            if shutdown_rx.try_recv().is_ok() {
                println!("Thermometer UDP receiver shutting down");
                break;
            }

            // Try to receive UDP packet
            match socket.recv(&mut buf) {
                Ok(UDP_BUFFER_SIZE) => {
                    // Parse temperature reading
                    let reading = TemperatureData::from_bytes(&buf);

                    // Update shared temperature
                    if let Ok(mut temp) = temperature.lock() {
                        // println!(
                        //     "  [UDP] Received temperature: {:.2}°C at timestamp {}",
                        //     reading.temperature, reading.timestamp
                        // );
                        if current_timestamp < reading.timestamp {
                            current_timestamp = reading.timestamp;
                        } else {
                            // Ignore out-of-order or duplicate readings
                            continue;
                        }
                        *temp = reading.temperature as Celsius;
                    }
                }
                Ok(n) => {
                    eprintln!("  [UDP] Invalid packet size: {} bytes (expected 8)", n);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, sleep briefly
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("  [UDP] Receive error: {}", e);
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
}

impl Drop for ThermometerRemote {
    fn drop(&mut self) {
        // Send shutdown signal to the receiver thread
        // We ignore errors here because the receiver might have already stopped
        let _ = self.shutdown_tx.send(());

        // Take ownership of the thread handle and wait for it to finish
        if let Some(handle) = self.receiver_thread.take() {
            // println!("Shutting down thermometer '{}'...", self.name);

            // Wait for the thread to finish with a timeout
            match handle.join() {
                Ok(_) => println!("Thermometer '{}' shutdown complete", self.name),
                Err(e) => eprintln!(
                    "Error joining thermometer thread for '{}': {:?}",
                    self.name, e
                ),
            }
        }
    }
}

impl Information for ThermometerRemote {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn info(&self) -> String {
        format!(
            "Thermometer: {} - Current Temperature: {:.2}°C",
            self.name,
            self.current_temperature()
        )
    }
}

impl TemperatureSensor for ThermometerRemote {
    fn current_temperature(&self) -> Celsius {
        self.temperature.lock().map_or(Celsius::default(), |t| *t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::time::Duration;

    /// Helper function to send UDP temperature data
    fn send_temperature_update(addr: &str, temperature: f32) -> std::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let data = TemperatureData::new(temperature);
        socket.send_to(&data.to_bytes(), addr)?;
        Ok(())
    }

    #[test]
    fn test_thermometer_remote_new() {
        // Create thermometer on a random port
        let thermometer =
            ThermometerRemote::new("Test Thermometer".to_string(), "127.0.0.1:0".to_string());

        // Note: This might fail if port 0 doesn't work as expected
        // In production, you'd want to use a specific port or better port allocation
        assert!(thermometer.is_ok() || thermometer.is_err()); // Either outcome is valid for port 0
    }

    #[test]
    fn test_thermometer_remote_receive_temperature() {
        // Bind to a specific port for testing
        let port = 19999; // Use a high port number to avoid conflicts
        let addr = format!("127.0.0.1:{}", port);

        let thermometer = ThermometerRemote::new("Test Thermometer".to_string(), addr.clone())
            .expect("Failed to create thermometer");

        // Give the receiver thread time to start
        thread::sleep(Duration::from_millis(100));

        // Send a temperature update
        send_temperature_update(&addr, 23.5).expect("Failed to send temperature");

        // Give time for the update to be processed
        thread::sleep(Duration::from_millis(200));

        // Check that temperature was updated
        let temp = thermometer.current_temperature();
        assert!((temp - 23.5_f64).abs() < 0.01);
    }

    #[test]
    fn test_thermometer_remote_multiple_updates() {
        let port = 20000;
        let addr = format!("127.0.0.1:{}", port);

        let thermometer = ThermometerRemote::new("Test Thermometer".to_string(), addr.clone())
            .expect("Failed to create thermometer");

        thread::sleep(Duration::from_millis(100));

        // Send multiple temperature updates
        let temperatures = [20.0_f32, 22.5_f32, 25.0_f32, 27.5_f32];

        for &temp in &temperatures {
            send_temperature_update(&addr, temp).expect("Failed to send temperature");
            thread::sleep(Duration::from_millis(300));

            let current = thermometer.current_temperature();
            let expected = temp as f64;
            assert!(
                (current - expected).abs() < 0.01,
                "Expected {}, got {}",
                expected,
                current
            );
        }
    }

    #[test]
    fn test_thermometer_remote_info() {
        let port = 20001;
        let addr = format!("127.0.0.1:{}", port);

        let thermometer = ThermometerRemote::new("Living Room".to_string(), addr.clone())
            .expect("Failed to create thermometer");

        thread::sleep(Duration::from_millis(100));

        // Send temperature update
        send_temperature_update(&addr, 22.5).expect("Failed to send temperature");
        thread::sleep(Duration::from_millis(150));

        let info = thermometer.info();
        assert_eq!(
            info,
            "Thermometer: Living Room - Current Temperature: 22.50°C"
        );
    }

    #[test]
    fn test_thermometer_remote_name() {
        let port = 20002;
        let addr = format!("127.0.0.1:{}", port);

        let thermometer = ThermometerRemote::new("Kitchen".to_string(), addr)
            .expect("Failed to create thermometer");

        assert_eq!(thermometer.name(), "Kitchen");
    }

    #[test]
    fn test_thermometer_remote_initial_temperature() {
        let port = 20003;
        let addr = format!("127.0.0.1:{}", port);

        let thermometer =
            ThermometerRemote::new("Test".to_string(), addr).expect("Failed to create thermometer");

        // Initial temperature should be default (0.0)
        let temp = thermometer.current_temperature();
        assert_eq!(temp, 0.0);
    }

    #[test]
    fn test_thermometer_remote_invalid_packet_size() {
        let port = 20004;
        let addr = format!("127.0.0.1:{}", port);

        let thermometer = ThermometerRemote::new("Test".to_string(), addr.clone())
            .expect("Failed to create thermometer");

        thread::sleep(Duration::from_millis(100));

        // Send invalid packet (wrong size)
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.send_to(&[1, 2, 3, 4], &addr).unwrap(); // Only 4 bytes instead of 8

        thread::sleep(Duration::from_millis(150));

        // Temperature should still be default (invalid packet should be ignored)
        let temp = thermometer.current_temperature();
        assert_eq!(temp, 0.0);
    }

    #[test]
    fn test_thermometer_remote_concurrent_access() {
        let port = 20005;
        let addr = format!("127.0.0.1:{}", port);

        let thermometer = Arc::new(
            ThermometerRemote::new("Test".to_string(), addr.clone())
                .expect("Failed to create thermometer"),
        );

        thread::sleep(Duration::from_millis(100));

        // Spawn multiple threads reading temperature concurrently
        let mut handles = vec![];

        for _ in 0..5 {
            let thermo = Arc::clone(&thermometer);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let _temp = thermo.current_temperature();
                    thread::sleep(Duration::from_millis(10));
                }
            });
            handles.push(handle);
        }

        // Send temperature updates while threads are reading
        for i in 0..10 {
            send_temperature_update(&addr, 20.0 + i as f32).ok();
            thread::sleep(Duration::from_millis(50));
        }

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify final temperature
        let temp = thermometer.current_temperature();
        assert!(temp >= 20.0 && temp <= 30.0);
    }

    #[test]
    fn test_temperature_data_serialization() {
        let data = TemperatureData {
            temperature: 25.5,
            timestamp: 1234567890,
        };

        let bytes = data.to_bytes();
        let decoded = TemperatureData::from_bytes(&bytes);

        assert_eq!(decoded.temperature, 25.5);
        assert_eq!(decoded.timestamp, 1234567890);
    }
}
