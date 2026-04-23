pub mod errors;
pub mod linak;
pub mod manager;
pub mod move_coord;

pub use errors::DeskError;
pub use manager::BleController;

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

/// Invariant: every write to a Linak controller is `WithoutResponse`. Using
/// WithResponse locks the controller up after 1-2 packets on some firmwares.
/// Shared by `manager` (session-level wake/stop writes) and `move_coord`
/// (move-loop tick writes).
pub(crate) async fn write_no_response(
    peripheral: &Peripheral,
    ch: &Characteristic,
    data: &[u8],
) -> Result<(), DeskError> {
    peripheral
        .write(ch, data, WriteType::WithoutResponse)
        .await?;
    Ok(())
}
