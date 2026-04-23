use serde::{ser::SerializeStruct, Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum DeskError {
    #[error("no bluetooth adapter available")]
    NoAdapter,

    #[error("bluetooth permission denied or powered off")]
    PermissionDenied,

    #[error("no matching desk found after scan")]
    NotFound,

    #[error("not connected to a desk")]
    NotConnected,

    #[error("required characteristic missing ({0})")]
    MissingCharacteristic(&'static str),

    #[error(
        "height {0} cm out of range [{}, {}]",
        super::linak::HEIGHT_MIN_CM,
        super::linak::HEIGHT_MAX_CM
    )]
    InvalidHeight(f32),

    #[error("move-to timed out before reaching target")]
    MoveTimeout,

    #[error("btleplug: {0}")]
    Btleplug(#[from] btleplug::Error),

    #[error("io: {0}")]
    Io(String),
}

impl DeskError {
    pub fn code(&self) -> &'static str {
        match self {
            DeskError::NoAdapter => "no_adapter",
            DeskError::PermissionDenied => "permission_denied",
            DeskError::NotFound => "not_found",
            DeskError::NotConnected => "not_connected",
            DeskError::MissingCharacteristic(_) => "missing_characteristic",
            DeskError::InvalidHeight(_) => "invalid_height",
            DeskError::MoveTimeout => "move_timeout",
            DeskError::Btleplug(_) => "btleplug",
            DeskError::Io(_) => "io",
        }
    }

    pub fn recoverable(&self) -> bool {
        !matches!(self, DeskError::NoAdapter | DeskError::PermissionDenied)
    }
}

impl Serialize for DeskError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("DeskError", 3)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        st.serialize_field("recoverable", &self.recoverable())?;
        st.end()
    }
}
