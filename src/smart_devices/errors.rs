#[derive(Debug)]
pub enum OutletRemoteError {
    NetworkError(std::io::Error),
    ProtocolError(String),
}

impl std::fmt::Display for OutletRemoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutletRemoteError::NetworkError(e) => write!(f, "Network error: {}", e),
            OutletRemoteError::ProtocolError(e) => write!(f, "Protocol error: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum OutletError {
    OutletRemoteError,
    UnknownError(String),
}

impl std::fmt::Display for OutletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutletError::OutletRemoteError => write!(f, "Outlet remote error occurred"),
            OutletError::UnknownError(e) => write!(f, "Unknown error: {}", e),
        }
    }
}
