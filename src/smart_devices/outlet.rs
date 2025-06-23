use super::types::Watt;
use crate::info::Information;

use std::fmt;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum OutletState {
    On,
    Off,
}

impl fmt::Display for OutletState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutletState::On => write!(f, "On"),
            OutletState::Off => write!(f, "Off"),
        }
    }
}

pub trait OutletDevice: Information {
    fn new(name: String, initial_state: OutletState, power_usage: Watt) -> Self;
    fn turn_on(&mut self);
    fn turn_off(&mut self);
    fn switch(&mut self);
    fn state(&self) -> OutletState;
    fn power_usage(&self) -> Watt;
}

#[derive(Debug, Clone)]
pub struct Outlet {
    name: String,
    state: OutletState,
    power_usage: Watt,
}

impl Information for Outlet {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn info(&self) -> String {
        let usage = OutletDevice::power_usage(self);
        format!(
            "Smart Outlet: {} - Current State: {}, Power Usage: {} Watt",
            self.name, self.state, usage
        )
    }
}

impl OutletDevice for Outlet {
    fn new(name: String, initial_state: OutletState, power_usage: Watt) -> Self {
        Outlet {
            name,
            state: initial_state,
            power_usage,
        }
    }

    fn turn_on(&mut self) {
        self.state = OutletState::On;
    }

    fn turn_off(&mut self) {
        self.state = OutletState::Off
    }

    fn switch(&mut self) {
        self.state = match self.state {
            OutletState::On => OutletState::Off,
            OutletState::Off => OutletState::On,
        };
    }

    fn state(&self) -> OutletState {
        self.state
    }

    fn power_usage(&self) -> Watt {
        match self.state {
            OutletState::On => self.power_usage,
            OutletState::Off => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlet_create_test() {
        let outlet = Outlet::new("Living Room Outlet".to_string(), OutletState::Off, 100);
        assert_eq!(outlet.power_usage(), 0);
        assert_eq!(outlet.state(), OutletState::Off);
        assert_eq!(outlet.name(), "Living Room Outlet");
        assert_eq!(
            outlet.info(),
            "Smart Outlet: Living Room Outlet - Current State: Off, Power Usage: 0 Watt"
        );
    }
    #[test]
    fn outlet_switch_test() {
        let mut outlet = Outlet::new("Living Room Outlet".to_string(), OutletState::Off, 100);
        assert_eq!(outlet.power_usage(), 0);
        assert_eq!(outlet.state(), OutletState::Off);
        assert_eq!(outlet.name(), "Living Room Outlet");
        outlet.switch();
        assert_eq!(outlet.power_usage(), 100);
        assert_eq!(outlet.state(), OutletState::On);
        assert_eq!(
            outlet.info(),
            "Smart Outlet: Living Room Outlet - Current State: On, Power Usage: 100 Watt"
        );
    }
    #[test]
    fn outlet_turn_on_off_test() {
        let mut outlet = Outlet::new("Living Room Outlet".to_string(), OutletState::Off, 100);
        assert_eq!(outlet.power_usage(), 0);
        assert_eq!(outlet.state(), OutletState::Off);
        assert_eq!(outlet.name(), "Living Room Outlet");
        outlet.turn_off();
        assert_eq!(outlet.power_usage(), 0);
        assert_eq!(outlet.state(), OutletState::Off);
        outlet.turn_on();
        assert_eq!(outlet.power_usage(), 100);
        assert_eq!(outlet.state(), OutletState::On);
        outlet.turn_off();
        assert_eq!(outlet.power_usage(), 0);
        assert_eq!(outlet.state(), OutletState::Off);
        assert_eq!(
            outlet.info(),
            "Smart Outlet: Living Room Outlet - Current State: Off, Power Usage: 0 Watt"
        );
    }
}
