use std::fmt;

pub type Watt = u64;
pub type Celsius = f64;
pub type Fahrenheit = f64;
pub type Kelvin = f64;

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

impl From<crate::protocols::outlet::OutletState> for OutletState {
    fn from(remote_state: crate::protocols::outlet::OutletState) -> Self {
        match remote_state {
            crate::protocols::outlet::OutletState::On => OutletState::On,
            crate::protocols::outlet::OutletState::Off => OutletState::Off,
        }
    }
}
