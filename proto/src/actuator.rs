use serde::{Deserialize, Serialize};

#[cfg(feature = "actuator")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, defmt::Format)]
#[repr(C)]
pub enum Request {
    Generate {
        rising_edges: u32,
        period_picos: u64,
    },
    WhoAreYou,
}

#[cfg(feature = "actuator")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, defmt::Format)]
#[repr(C)]
pub enum Response {
    StartedGenerating,
    FailedGenerating,
    FinishedGenerating,
}

#[cfg(all(test, feature = "actuator"))]
mod test {
    use core::ops::Deref;
    use heapless::Vec;
    use postcard::{from_bytes, to_vec};

    use super::*;

    #[test]
    fn request() {
        let request = Request::Generate {
            rising_edges: 5,
            period_picos: 2,
        };
        let output: Vec<u8, 13> = to_vec(&request).unwrap();
        let back: Request = from_bytes(output.deref()).unwrap();
        assert_eq!(back, request);
    }
}
