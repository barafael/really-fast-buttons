use serde::{Deserialize, Serialize};

#[cfg(feature = "actuator")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, defmt::Format)]
#[repr(C)]
pub enum Message {
    Request {
        rising_edges: u64,
        period_picos: u64,
    },
}
