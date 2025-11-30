#[derive(Debug)]
pub enum SimulatorErrors {
    Thermometer(String),
    Outlet(String),
}

impl std::fmt::Display for SimulatorErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulatorErrors::Thermometer(e) => write!(f, "Thermometer simulator error: {}", e),
            SimulatorErrors::Outlet(e) => write!(f, "Outlet simulator error: {}", e),
        }
    }
}
