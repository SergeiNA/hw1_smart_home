use crate::protocols::thermometer::TemperatureData;
use crate::simulators::errors::SimulatorErrors;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const DEFAULT_TEMPERATURE_CELSIUS: f32 = 22.0;

/// Temperature generation pattern
#[derive(Debug, Clone, Copy)]
pub enum TemperaturePattern {
    /// Random walk within range
    RandomWalk { min: f32, max: f32, step: f32 },
    /// Sine wave pattern
    SineWave {
        center: f32,
        amplitude: f32,
        period_secs: f32,
    },
    /// Fixed constant value
    Constant(f32),
}

impl Default for TemperaturePattern {
    fn default() -> Self {
        TemperaturePattern::Constant(DEFAULT_TEMPERATURE_CELSIUS)
    }
}

#[derive(Debug, Clone)]
pub struct ThermometerSimulatorConfig {
    pub name: String,
    /// Target UDP address to send readings to (e.g., "127.0.0.1:9001")
    pub target_addr: String,
    /// How often to send readings
    pub send_interval: Duration,
    /// Temperature generation pattern
    pub pattern: TemperaturePattern,
}

impl ThermometerSimulatorConfig {
    pub fn new(name: &str, target_addr: impl Into<String>, send_interval: Duration) -> Self {
        Self {
            name: name.into(),
            target_addr: target_addr.into(),
            send_interval,
            pattern: TemperaturePattern::default(),
        }
    }

    pub fn with_pattern(mut self, pattern: TemperaturePattern) -> Self {
        self.pattern = pattern;
        self
    }
}

/// Temperature value generator
struct TemperatureGenerator {
    pattern: TemperaturePattern,
    current: f32,
    iteration: u64,
}

impl TemperatureGenerator {
    fn new(pattern: TemperaturePattern) -> Self {
        let current = match pattern {
            TemperaturePattern::RandomWalk { min, max, .. } => (min + max) / 2.0,
            TemperaturePattern::SineWave { center, .. } => center,
            TemperaturePattern::Constant(val) => val,
        };

        Self {
            pattern,
            current,
            iteration: 0,
        }
    }

    fn next(&mut self) -> f32 {
        self.iteration += 1;

        match self.pattern {
            TemperaturePattern::RandomWalk { min, max, step } => {
                use rand::Rng;
                let mut rng = rand::rng();
                let change = rng.random_range(-step..=step);
                self.current = (self.current + change).clamp(min, max);
                self.current
            }
            TemperaturePattern::SineWave {
                center,
                amplitude,
                period_secs,
            } => {
                let t = self.iteration as f32 / period_secs;
                center + amplitude * (t * 2.0 * std::f32::consts::PI).sin()
            }
            TemperaturePattern::Constant(val) => val,
        }
    }
}

/// Handle to a running thermometer simulator
/// When dropped, the simulator is automatically stopped
pub struct ThermometerSimulator {
    target_addr: String,
    sender_thread: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl ThermometerSimulator {
    /// Spawn a new thermometer simulator
    ///
    /// Returns a handle to the simulator that will automatically
    /// clean up when dropped.
    pub fn spawn(config: ThermometerSimulatorConfig) -> Result<Self, SimulatorErrors> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| SimulatorErrors::Thermometer(e.to_string()))?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);
        let target_addr = config.target_addr.clone();
        let sender_thread = thread::spawn(move || {
            Self::run_sender(socket, config, shutdown_clone);
        });

        Ok(Self {
            target_addr,
            sender_thread: Some(sender_thread),
            shutdown,
        })
    }

    /// Run the UDP sender loop
    fn run_sender(
        socket: UdpSocket,
        config: ThermometerSimulatorConfig,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut generator = TemperatureGenerator::new(config.pattern);

        while !shutdown.load(Ordering::Relaxed) {
            // Generate next temperature
            let temp = generator.next();

            // Create temperature reading
            let reading = TemperatureData::new(temp);
            let buf = reading.to_bytes();

            // Send UDP packet
            match socket.send_to(&buf, &config.target_addr) {
                Ok(_) => {
                    // println!(
                    //     "[Thermometer Simulator] Sent temperature {:.2}°C to {}",
                    //     temp, config.target_addr
                    // );
                }
                Err(e) => {
                    eprintln!("[Thermometer Simulator] Failed to send: {}", e);
                }
            }

            // Wait before next reading
            thread::sleep(config.send_interval);
        }
    }

    /// Gracefully stop the simulator
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    pub fn target_address(&self) -> &str {
        &self.target_addr
    }
}

impl Drop for ThermometerSimulator {
    fn drop(&mut self) {
        // Signal shutdown
        self.stop();

        // Wait for thread to finish
        if let Some(handle) = self.sender_thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_devices::thermometer_remote::ThermometerRemote;
    use crate::smart_devices::{Celsius, TemperatureSensor};
    use std::time::Duration;

    #[test]
    fn thermometer_simulator_config_default_pattern() {
        let config = ThermometerSimulatorConfig::new(
            "Test Thermometer",
            "127.0.0.1:9001",
            Duration::from_secs(1),
        );
        assert_eq!(config.name, "Test Thermometer");
        assert_eq!(config.target_addr, "127.0.0.1:9001");
        assert_eq!(config.send_interval, Duration::from_secs(1));
        match config.pattern {
            TemperaturePattern::Constant(val) => assert_eq!(val, 22.0),
            _ => panic!("Default pattern should be Constant(22.0)"),
        }
    }

    #[test]
    fn thermometer_simulator_config_with_pattern() {
        let config =
            ThermometerSimulatorConfig::new("Test", "127.0.0.1:9002", Duration::from_secs(2))
                .with_pattern(TemperaturePattern::RandomWalk {
                    min: 10.0,
                    max: 30.0,
                    step: 0.5,
                });
        match config.pattern {
            TemperaturePattern::RandomWalk { min, max, step } => {
                assert_eq!(min, 10.0);
                assert_eq!(max, 30.0);
                assert_eq!(step, 0.5);
            }
            _ => panic!("Pattern should be RandomWalk"),
        }
    }

    #[test]
    fn temperature_generator_constant() {
        let mut temperature_gen = TemperatureGenerator::new(TemperaturePattern::Constant(25.0));
        for _ in 0..5 {
            assert_eq!(temperature_gen.next(), 25.0);
        }
    }

    #[test]
    fn temperature_generator_sinewave() {
        let mut temperature_gen = TemperatureGenerator::new(TemperaturePattern::SineWave {
            center: 20.0,
            amplitude: 5.0,
            period_secs: 10.0,
        });
        let t1 = temperature_gen.next();
        let t2 = temperature_gen.next();
        assert!(t1 >= 15.0 && t1 <= 25.0);
        assert!(t2 >= 15.0 && t2 <= 25.0);
    }

    #[test]
    fn thermometer_simulator_create_test() {
        let config = ThermometerSimulatorConfig::new(
            "Test Thermometer",
            "127.0.0.1:9003",
            Duration::from_millis(10),
        );
        let sim = ThermometerSimulator::spawn(config);
        assert!(sim.is_ok());
        let sim = sim.unwrap();
        sim.stop();
    }

    #[test]
    fn thermometer_simulator_config_default_pattern_with_remote_thermometer() {
        let addr = "127.0.0.1:9011";
        let config =
            ThermometerSimulatorConfig::new("Test Thermometer", addr, Duration::from_millis(100));
        let sim = ThermometerSimulator::spawn(config);
        assert!(sim.is_ok());

        let thermometer_remote =
            ThermometerRemote::new("Bedroom thermometer".to_string(), addr.to_string());
        assert!(thermometer_remote.is_ok());
        let thermometer_remote = thermometer_remote.unwrap();

        for _ in 0..5 {
            // Allow some time for the simulator to send data
            std::thread::sleep(std::time::Duration::from_millis(150));

            let temp = thermometer_remote.current_temperature();
            assert_eq!(temp, DEFAULT_TEMPERATURE_CELSIUS as Celsius);
        }
        sim.unwrap().stop();
    }

    #[test]
    fn thermometer_simulator_config_sinewave_pattern_with_remote_thermometer() {
        let addr = "127.0.0.1:9012";
        let config =
            ThermometerSimulatorConfig::new("Test Thermometer", addr, Duration::from_millis(100))
                .with_pattern(TemperaturePattern::SineWave {
                    center: 20.0,
                    amplitude: 5.0,
                    period_secs: 10.0,
                });
        let sim = ThermometerSimulator::spawn(config);
        assert!(sim.is_ok());

        let thermometer_remote =
            ThermometerRemote::new("Bedroom thermometer".to_string(), addr.to_string());
        assert!(thermometer_remote.is_ok());
        let thermometer_remote = thermometer_remote.unwrap();

        let mut prev_temp: Celsius = 0.;
        for _ in 0..5 {
            // Allow some time for the simulator to send data
            std::thread::sleep(std::time::Duration::from_millis(300));

            let temp = thermometer_remote.current_temperature();
            assert!(temp >= 15.0 as Celsius && temp <= 25.0 as Celsius);
            assert_ne!(temp, prev_temp);
            prev_temp = temp;
        }
        sim.unwrap().stop();
    }

    #[test]
    fn thermometer_simulator_config_random_walk_pattern_with_remote_thermometer() {
        let addr = "127.0.0.1:9013";
        let config =
            ThermometerSimulatorConfig::new("Test Thermometer", addr, Duration::from_millis(100))
                .with_pattern(TemperaturePattern::RandomWalk {
                    min: 25.0,
                    step: 1.0,
                    max: 35.0,
                });
        let sim = ThermometerSimulator::spawn(config);
        assert!(sim.is_ok());

        let thermometer_remote =
            ThermometerRemote::new("Bedroom thermometer".to_string(), addr.to_string());
        assert!(thermometer_remote.is_ok());
        let thermometer_remote = thermometer_remote.unwrap();

        let mut prev_temp: Celsius = 0.;
        for _ in 0..5 {
            // Allow some time for the simulator to send data
            std::thread::sleep(std::time::Duration::from_millis(300));

            let temp = thermometer_remote.current_temperature();
            assert!(temp >= 25.0 as Celsius && temp <= 35.0 as Celsius);
            assert_ne!(temp, prev_temp);
            prev_temp = temp;
        }
        sim.unwrap().stop();
    }

    #[test]
    fn thermometer_simulator_with_remote_thermometer_test_after_stop() {
        let addr = "127.0.0.1:9014";
        let config =
            ThermometerSimulatorConfig::new("Test Thermometer", addr, Duration::from_millis(50))
                .with_pattern(TemperaturePattern::RandomWalk {
                    min: 25.0,
                    step: 1.0,
                    max: 35.0,
                });
        let sim = ThermometerSimulator::spawn(config);
        assert!(sim.is_ok());

        let thermometer_remote =
            ThermometerRemote::new("Bedroom thermometer".to_string(), addr.to_string());
        assert!(thermometer_remote.is_ok());
        let thermometer_remote = thermometer_remote.unwrap();

        thread::sleep(Duration::from_millis(150));
        sim.unwrap().stop();
        thread::sleep(Duration::from_millis(150));
        let prev_temp = thermometer_remote.current_temperature();
        assert!(prev_temp >= 25.0 as Celsius);

        for _ in 0..5 {
            // Allow some time for the simulator to send data
            std::thread::sleep(Duration::from_millis(150));
            let temp = thermometer_remote.current_temperature();
            assert!((prev_temp - temp).abs() < 0.01);
        }
    }
}
