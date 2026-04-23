use serde::{Deserialize, Serialize};

pub const EVT_HEIGHT: &str = "desk://height";
pub const EVT_CONNECTION: &str = "desk://connection";
pub const EVT_ERROR: &str = "desk://error";
/// Fired `true` when the popover becomes visible, `false` when it hides.
/// `useAutoSession` uses this to pause/resume the BLE notification stream so
/// the desk controller display idles while the UI is off-screen.
pub const EVT_PANEL_VISIBILITY: &str = "desk://panel-visibility";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeightUpdate {
    pub cm: f32,
    pub mm: u16,
    pub speed: i16,
    pub moving: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum ConnectionUpdate {
    Disconnected,
    Scanning,
    Connecting { device: String },
    Connected { device: String, address: String },
    Reconnecting { attempt: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeskErrorPayload {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}
