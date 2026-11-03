use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverError {
    UnsupportedDevice { vendor_id: u16, product_id: u16 },
    DeviceUnavailable(String),
    PermissionDenied(String),
    DescriptorUnavailable(String),
    MalformedReport(String),
    InvalidPacketLength { expected: String, actual: usize },
    TransportDisabled(&'static str),
    UnsupportedOperation(&'static str),
    Io(String),
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDevice {
                vendor_id,
                product_id,
            } => write!(f, "unsupported USB device {vendor_id:04x}:{product_id:04x}"),
            Self::DeviceUnavailable(id) => write!(f, "device unavailable: {id}"),
            Self::PermissionDenied(detail) => write!(f, "permission denied: {detail}"),
            Self::DescriptorUnavailable(detail) => write!(f, "descriptor unavailable: {detail}"),
            Self::MalformedReport(detail) => write!(f, "malformed report: {detail}"),
            Self::InvalidPacketLength { expected, actual } => write!(
                f,
                "invalid packet length: expected {expected}, got {actual}"
            ),
            Self::TransportDisabled(operation) => write!(f, "transport disabled: {operation}"),
            Self::UnsupportedOperation(operation) => {
                write!(f, "unsupported operation: {operation}")
            }
            Self::Io(detail) => write!(f, "I/O error: {detail}"),
        }
    }
}

impl std::error::Error for DriverError {}

impl From<crate::capabilities::ParseError> for DriverError {
    fn from(value: crate::capabilities::ParseError) -> Self {
        Self::MalformedReport(value.to_string())
    }
}

impl From<std::io::Error> for DriverError {
    fn from(value: std::io::Error) -> Self {
        if value.kind() == std::io::ErrorKind::PermissionDenied {
            Self::PermissionDenied(value.to_string())
        } else {
            Self::Io(value.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::ParseError;

    #[test]
    fn parser_errors_convert_to_malformed_report() {
        let error = DriverError::from(ParseError {
            offset: 3,
            message: "truncated item".into(),
        });
        assert!(matches!(error, DriverError::MalformedReport(_)));
        assert!(error.to_string().contains("descriptor offset 0x3"));
    }

    #[test]
    fn errors_have_stable_human_readable_messages() {
        assert!(DriverError::UnsupportedDevice {
            vendor_id: 0x3151,
            product_id: 0x9999
        }
        .to_string()
        .contains("3151:9999"));
        assert!(DriverError::InvalidPacketLength {
            expected: "64".into(),
            actual: 63
        }
        .to_string()
        .contains("got 63"));
        assert!(DriverError::TransportDisabled("feature get")
            .to_string()
            .contains("transport disabled"));
    }

    #[test]
    fn permission_io_errors_convert_to_permission_denied() {
        let error = DriverError::from(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        assert!(matches!(error, DriverError::PermissionDenied(_)));
    }
}
