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

#[cfg(test)]
mod test {
    use core::ops::Deref;
    use heapless::Vec;
    use postcard::{from_bytes, to_vec};

    use super::*;

    #[test]
    fn request() {
        let request = Message::Request {
            rising_edges: 5,
            period_picos: 2,
        };
        let output: Vec<u8, 17> = to_vec(&request).unwrap();
        let back: Message = from_bytes(output.deref()).unwrap();
        assert_eq!(back, request);
    }
}
