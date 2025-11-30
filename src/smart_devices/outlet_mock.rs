use super::types::OutletState;
use super::types::Watt;
use crate::traits::Information;

use crate::smart_devices::errors::OutletRemoteError;

pub trait OutletDevice: Information {
    fn turn_on(&mut self) -> Result<(), OutletRemoteError>;
    fn turn_off(&mut self) -> Result<(), OutletRemoteError>;
    fn switch(&mut self) -> Result<(), OutletRemoteError>;
    fn state(&self) -> Result<OutletState, OutletRemoteError>;
    fn power_usage(&self) -> Result<Watt, OutletRemoteError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutletMock {
    name: String,
    state: OutletState,
    power_usage: Watt,
}

impl OutletMock {
    pub fn new(name: String, initial_state: OutletState, power_usage: Watt) -> Self {
        OutletMock {
            name,
            state: initial_state,
            power_usage,
        }
    }
}

impl Information for OutletMock {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn info(&self) -> String {
        // For local outlet device, we can get power usage directly
        let power = self.power_usage().unwrap_or_default();
        format!(
            "Smart Outlet: {} - Current State: {}, Power Usage: {} Watt",
            self.name, self.state, power
        )
    }
}

impl OutletDevice for OutletMock {
    fn turn_on(&mut self) -> Result<(), OutletRemoteError> {
        self.state = OutletState::On;
        Ok(())
    }

    fn turn_off(&mut self) -> Result<(), OutletRemoteError> {
        self.state = OutletState::Off;
        Ok(())
    }

    fn switch(&mut self) -> Result<(), OutletRemoteError> {
        self.state = match self.state {
            OutletState::On => OutletState::Off,
            OutletState::Off => OutletState::On,
        };
        Ok(())
    }

    fn state(&self) -> Result<OutletState, OutletRemoteError> {
        Ok(self.state)
    }

    fn power_usage(&self) -> Result<Watt, OutletRemoteError> {
        let power = match self.state {
            OutletState::On => self.power_usage,
            OutletState::Off => 0,
        };
        Ok(power)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlet_create_test() {
        let outlet = OutletMock::new("Living Room Outlet".to_string(), OutletState::Off, 100);
        assert_eq!(outlet.power_usage().unwrap(), 0);
        assert_eq!(outlet.state().unwrap(), OutletState::Off);
        assert_eq!(outlet.name(), "Living Room Outlet");
        assert_eq!(
            outlet.info(),
            "Smart Outlet: Living Room Outlet - Current State: Off, Power Usage: 0 Watt"
        );
    }
    #[test]
    fn outlet_switch_test() {
        let mut outlet = OutletMock::new("Living Room Outlet".to_string(), OutletState::Off, 100);
        assert_eq!(outlet.power_usage().unwrap(), 0);
        assert_eq!(outlet.state().unwrap(), OutletState::Off);
        assert_eq!(outlet.name(), "Living Room Outlet");
        outlet.switch().expect("Smart Outlet - Switch failed");
        assert_eq!(outlet.power_usage().unwrap(), 100);
        assert_eq!(outlet.state().unwrap(), OutletState::On);
        assert_eq!(
            outlet.info(),
            "Smart Outlet: Living Room Outlet - Current State: On, Power Usage: 100 Watt"
        );
    }
    #[test]
    fn outlet_turn_on_off_test() {
        let mut outlet = OutletMock::new("Living Room Outlet".to_string(), OutletState::Off, 100);
        assert_eq!(outlet.power_usage().unwrap(), 0);
        assert_eq!(outlet.state().unwrap(), OutletState::Off);
        assert_eq!(outlet.name(), "Living Room Outlet");
        outlet.turn_off().unwrap();
        assert_eq!(outlet.power_usage().unwrap(), 0);
        assert_eq!(outlet.state().unwrap(), OutletState::Off);
        outlet.turn_on().unwrap();
        assert_eq!(outlet.power_usage().unwrap(), 100);
        assert_eq!(outlet.state().unwrap(), OutletState::On);
        outlet.turn_off().unwrap();
        assert_eq!(outlet.power_usage().unwrap(), 0);
        assert_eq!(outlet.state().unwrap(), OutletState::Off);
        assert_eq!(
            outlet.info(),
            "Smart Outlet: Living Room Outlet - Current State: Off, Power Usage: 0 Watt"
        );
    }
}
