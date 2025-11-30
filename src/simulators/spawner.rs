use crate::simulators::errors::SimulatorErrors;
use crate::simulators::outlet::{OutletSimulator, OutletSimulatorConfig};
use crate::simulators::thermometer::{ThermometerSimulator, ThermometerSimulatorConfig};
use std::collections::HashMap;

#[derive(Default)]
pub struct DeviceSimulatorSpawner {
    outlet_simulators: HashMap<String, OutletSimulator>,
    thermometer_simulators: HashMap<String, ThermometerSimulator>,
}

impl DeviceSimulatorSpawner {
    pub fn spawn_outlet_simulator(
        &mut self,
        name: String,
        config: OutletSimulatorConfig,
    ) -> Result<String, SimulatorErrors> {
        let simulator = OutletSimulator::spawn(config);
        match simulator {
            Ok(sim) => {
                let address = sim.address().to_string();
                self.outlet_simulators.insert(name.clone(), sim);
                Ok(address)
            }
            Err(e) => Err(SimulatorErrors::Outlet(format!(
                "Failed to spawn outlet simulator: {}",
                e
            ))),
        }
    }

    // TODO: spawn methods should use configs instead of individual parameters
    // Return address of spawned simulator
    pub fn spawn_thermometer_simulator(
        &mut self,
        name: String,
        config: ThermometerSimulatorConfig,
    ) -> Result<String, SimulatorErrors> {
        let simulator = ThermometerSimulator::spawn(config);
        match simulator {
            Ok(simulator) => {
                let target_addr = simulator.target_address().to_string();
                self.thermometer_simulators.insert(name, simulator);
                Ok(target_addr)
            }
            Err(e) => Err(SimulatorErrors::Thermometer(format!(
                "Failed to spawn thermometer simulator: {}",
                e
            ))),
        }
    }

    #[allow(dead_code)]
    pub fn get_outlet_simulator(&self, name: &str) -> Option<&OutletSimulator> {
        self.outlet_simulators.get(name)
    }

    #[allow(dead_code)]
    pub fn get_thermometer_simulator(&self, name: &str) -> Option<&ThermometerSimulator> {
        self.thermometer_simulators.get(name)
    }

    #[allow(dead_code)]
    pub fn list_outlet_simulators(&self) -> Vec<String> {
        self.outlet_simulators.keys().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn list_thermometer_simulators(&self) -> Vec<String> {
        self.thermometer_simulators.keys().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.outlet_simulators.is_empty() && self.thermometer_simulators.is_empty()
    }

    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.outlet_simulators.len() + self.thermometer_simulators.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulators::thermometer::TemperaturePattern;
    use crate::smart_devices::Watt;
    use crate::smart_devices::outlet_remote::OutletRemote;
    use crate::smart_devices::thermometer_remote::ThermometerRemote;
    use crate::smart_devices::types::OutletState;
    use crate::smart_devices::{OutletDevice, TemperatureSensor};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn spawn_outlet_simulator_test() {
        let mut spawner = DeviceSimulatorSpawner::default();
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 100 as Watt);
        let result = spawner.spawn_outlet_simulator("TestOutlet".to_string(), config);
        assert!(result.is_ok());
    }

    #[test]
    fn spawn_thermometer_simulator_test() {
        let mut spawner = DeviceSimulatorSpawner::default();
        let config = ThermometerSimulatorConfig::new(
            "TestThermometer",
            "127.0.0.1:0".to_string(),
            Duration::from_millis(100),
        )
        .with_pattern(TemperaturePattern::Constant(22.5));

        let result = spawner.spawn_thermometer_simulator("TestThermometer".to_string(), config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_outlet_simulator_with_remote_connection() {
        let mut spawner = DeviceSimulatorSpawner::default();

        // Spawn outlet simulator on a random port
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 150 as Watt);
        let addr = spawner
            .spawn_outlet_simulator("TestOutlet".to_string(), config)
            .expect("Failed to spawn outlet simulator");

        // Give simulator time to start
        thread::sleep(Duration::from_millis(100));

        // Connect remote outlet to the simulator
        let mut remote = OutletRemote::new("RemoteOutlet".to_string(), addr)
            .expect("Failed to connect remote outlet");

        // Test getting initial state (should be Off by default)
        let state = remote.state().expect("Failed to get state");
        assert_eq!(state, OutletState::Off);

        // Test turning on
        remote.turn_on().expect("Failed to turn on");
        thread::sleep(Duration::from_millis(50));
        let state = remote.state().expect("Failed to get state");
        assert_eq!(state, OutletState::On);

        // Test getting power usage when on
        let power = remote.power_usage().expect("Failed to get power");
        assert_eq!(power, 150);

        // Test turning off
        remote.turn_off().expect("Failed to turn off");
        thread::sleep(Duration::from_millis(50));
        let state = remote.state().expect("Failed to get state");
        assert_eq!(state, OutletState::Off);

        // Test power usage when off (should be 0)
        let power = remote.power_usage().expect("Failed to get power");
        assert_eq!(power, 0);
    }

    #[test]
    fn test_outlet_simulator_switch_functionality() {
        let mut spawner = DeviceSimulatorSpawner::default();

        let config = OutletSimulatorConfig::new("127.0.0.1:0", 200 as Watt);
        let addr = spawner
            .spawn_outlet_simulator("SwitchOutlet".to_string(), config)
            .expect("Failed to spawn outlet simulator");

        thread::sleep(Duration::from_millis(100));

        let mut remote = OutletRemote::new("SwitchRemote".to_string(), addr)
            .expect("Failed to connect remote outlet");

        // Initially off
        let state = remote.state().expect("Failed to get state");
        assert_eq!(state, OutletState::Off);

        // Switch should turn it on
        remote.switch().expect("Failed to switch");
        thread::sleep(Duration::from_millis(50));
        let state = remote.state().expect("Failed to get state");
        assert_eq!(state, OutletState::On);

        // Switch again should turn it off
        remote.switch().expect("Failed to switch");
        thread::sleep(Duration::from_millis(50));
        let state = remote.state().expect("Failed to get state");
        assert_eq!(state, OutletState::Off);
    }

    #[test]
    fn test_thermometer_simulator_with_remote_connection() {
        let mut spawner = DeviceSimulatorSpawner::default();

        // Create a remote thermometer first to get a UDP port
        let _ = ThermometerRemote::new("RemoteThermo".to_string(), "127.0.0.1:0".to_string())
            .expect("Failed to create remote thermometer");

        // Get the actual UDP address (this is tricky with UDP, so we'll use a fixed port for testing)
        let udp_addr = "127.0.0.1:30001".to_string();

        // Create the remote on the specific port
        let thermo_remote = ThermometerRemote::new("RemoteThermo".to_string(), udp_addr.clone())
            .expect("Failed to create remote thermometer");

        thread::sleep(Duration::from_millis(100));

        // Spawn thermometer simulator that sends to this address
        let config = ThermometerSimulatorConfig::new(
            "TestThermometer",
            udp_addr,
            Duration::from_millis(100),
        )
        .with_pattern(TemperaturePattern::Constant(25.5));

        let _ = spawner
            .spawn_thermometer_simulator("TestThermometer".to_string(), config)
            .expect("Failed to spawn thermometer simulator");

        // Wait for at least one temperature update to be sent
        thread::sleep(Duration::from_millis(1500));

        // Check that the remote received the temperature
        let temp = thermo_remote.current_temperature();
        assert!((temp - 25.5).abs() < 0.1, "Expected ~25.5, got {}", temp);
    }

    #[test]
    fn test_thermometer_simulator_sine_wave_pattern() {
        let mut spawner = DeviceSimulatorSpawner::default();

        let udp_addr = "127.0.0.1:30002".to_string();

        let thermo_remote = ThermometerRemote::new("RemoteThermo".to_string(), udp_addr.clone())
            .expect("Failed to create remote thermometer");

        thread::sleep(Duration::from_millis(100));

        // Spawn with sine wave pattern
        let config = ThermometerSimulatorConfig::new(
            "SineWaveThermometer",
            udp_addr,
            Duration::from_millis(100),
        )
        .with_pattern(TemperaturePattern::SineWave {
            center: 20.0,
            amplitude: 5.0,
            period_secs: 10.0,
        });

        let _simulator = spawner
            .spawn_thermometer_simulator("SineWaveThermometer".to_string(), config)
            .expect("Failed to spawn thermometer simulator");

        // Wait for updates and check that temperature is within expected range
        thread::sleep(Duration::from_millis(1500));

        let temp = thermo_remote.current_temperature();
        // Temperature should be between base - amplitude and base + amplitude
        assert!(
            temp >= 15.0 && temp <= 25.0,
            "Temperature {} out of expected range [15.0, 25.0]",
            temp
        );
    }

    #[test]
    fn test_multiple_outlet_simulators_concurrent() {
        let mut spawner = DeviceSimulatorSpawner::default();

        // Spawn multiple outlet simulators and collect their addresses
        let config1 = OutletSimulatorConfig::new("127.0.0.1:0", 100 as Watt);
        let addr1 = spawner
            .spawn_outlet_simulator("Outlet1".to_string(), config1)
            .expect("Failed to spawn outlet 1");

        let config2 = OutletSimulatorConfig::new("127.0.0.1:0", 200 as Watt);
        let addr2 = spawner
            .spawn_outlet_simulator("Outlet2".to_string(), config2)
            .expect("Failed to spawn outlet 2");

        let config3 = OutletSimulatorConfig::new("127.0.0.1:0", 300 as Watt);
        let addr3 = spawner
            .spawn_outlet_simulator("Outlet3".to_string(), config3)
            .expect("Failed to spawn outlet 3");

        thread::sleep(Duration::from_millis(100));

        // Connect remote outlets to each simulator
        let mut remote1 =
            OutletRemote::new("Remote1".to_string(), addr1).expect("Failed to connect remote 1");
        let mut remote2 =
            OutletRemote::new("Remote2".to_string(), addr2).expect("Failed to connect remote 2");
        let mut remote3 =
            OutletRemote::new("Remote3".to_string(), addr3).expect("Failed to connect remote 3");

        // Turn on all outlets
        remote1.turn_on().expect("Failed to turn on outlet 1");
        remote2.turn_on().expect("Failed to turn on outlet 2");
        remote3.turn_on().expect("Failed to turn on outlet 3");

        thread::sleep(Duration::from_millis(100));

        // Verify all are on with correct power
        assert_eq!(remote1.state().unwrap(), OutletState::On);
        assert_eq!(remote1.power_usage().unwrap(), 100);

        assert_eq!(remote2.state().unwrap(), OutletState::On);
        assert_eq!(remote2.power_usage().unwrap(), 200);

        assert_eq!(remote3.state().unwrap(), OutletState::On);
        assert_eq!(remote3.power_usage().unwrap(), 300);
    }

    #[test]
    fn test_multiple_thermometer_simulators_concurrent() {
        let mut spawner = DeviceSimulatorSpawner::default();

        let udp_addr1 = "127.0.0.1:30010".to_string();
        let udp_addr2 = "127.0.0.1:30011".to_string();

        // Create remote thermometers
        let thermo1 = ThermometerRemote::new("Remote1".to_string(), udp_addr1.clone())
            .expect("Failed to create remote 1");
        let thermo2 = ThermometerRemote::new("Remote2".to_string(), udp_addr2.clone())
            .expect("Failed to create remote 2");

        thread::sleep(Duration::from_millis(100));

        // Spawn multiple thermometer simulators
        let config1 =
            ThermometerSimulatorConfig::new("Thermo1", udp_addr1, Duration::from_millis(100))
                .with_pattern(TemperaturePattern::Constant(20.0));

        spawner
            .spawn_thermometer_simulator("Thermo1".to_string(), config1)
            .expect("Failed to spawn thermo 1");

        let config2 =
            ThermometerSimulatorConfig::new("Thermo2", udp_addr2, Duration::from_millis(100))
                .with_pattern(TemperaturePattern::Constant(30.0));

        spawner
            .spawn_thermometer_simulator("Thermo2".to_string(), config2)
            .expect("Failed to spawn thermo 2");

        // Wait for updates
        thread::sleep(Duration::from_millis(1500));

        // Verify each remote received correct temperature
        let temp1 = thermo1.current_temperature();
        let temp2 = thermo2.current_temperature();

        assert!((temp1 - 20.0).abs() < 0.1, "Expected ~20.0, got {}", temp1);
        assert!((temp2 - 30.0).abs() < 0.1, "Expected ~30.0, got {}", temp2);
    }

    #[test]
    fn test_spawner_maintains_multiple_devices() {
        let mut spawner = DeviceSimulatorSpawner::default();

        // Spawn multiple devices of both types
        let config1 = OutletSimulatorConfig::new("127.0.0.1:0", 100);
        spawner
            .spawn_outlet_simulator("Outlet1".to_string(), config1)
            .expect("Failed to spawn outlet 1");

        let config2 = OutletSimulatorConfig::new("127.0.0.1:0", 200);
        spawner
            .spawn_outlet_simulator("Outlet2".to_string(), config2)
            .expect("Failed to spawn outlet 2");

        let thermo_config1 = ThermometerSimulatorConfig::new(
            "Thermo1",
            "127.0.0.1:30020".to_string(),
            Duration::from_millis(100),
        )
        .with_pattern(TemperaturePattern::Constant(22.0));

        spawner
            .spawn_thermometer_simulator("Thermo1".to_string(), thermo_config1)
            .expect("Failed to spawn thermo 1");

        let thermo_config2 = ThermometerSimulatorConfig::new(
            "Thermo2",
            "127.0.0.1:30021".to_string(),
            Duration::from_millis(100),
        )
        .with_pattern(TemperaturePattern::Constant(23.0));

        spawner
            .spawn_thermometer_simulator("Thermo2".to_string(), thermo_config2)
            .expect("Failed to spawn thermo 2");

        // Verify spawner maintains references to all devices
        assert_eq!(spawner.outlet_simulators.len(), 2);
        assert_eq!(spawner.thermometer_simulators.len(), 2);
    }

    #[test]
    fn test_outlet_simulator_connection_refused() {
        // Try to connect to a port that doesn't have a simulator
        let result = OutletRemote::new("TestOutlet".to_string(), "127.0.0.1:59999".to_string());
        assert!(result.is_err(), "Expected connection to fail");
    }

    #[test]
    fn test_spawner_duplicate_names() {
        let mut spawner = DeviceSimulatorSpawner::default();

        // Spawn first outlet
        let config1 = OutletSimulatorConfig::new("127.0.0.1:0", 100);
        let result1 = spawner.spawn_outlet_simulator("DuplicateOutlet".to_string(), config1);
        assert!(result1.is_ok());

        // Spawn second outlet with same name (should replace the first)
        let config2 = OutletSimulatorConfig::new("127.0.0.1:0", 200);
        let result2 = spawner.spawn_outlet_simulator("DuplicateOutlet".to_string(), config2);
        assert!(result2.is_ok());

        // Should only have one outlet with this name
        assert_eq!(spawner.outlet_simulators.len(), 1);
    }
}
