#![cfg_attr(not(test), no_std)]

use serde::{Deserialize, Serialize};

pub mod error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, defmt::Format)]
#[repr(C)]
pub enum Message {
    Request,
    Response(u64),
}

#[cfg(test)]
mod test {
    use core::ops::Deref;
    use heapless::Vec;
    use postcard::{from_bytes, to_vec};

    use super::*;

    #[test]
    fn config_response() {
        let response = Message::Response(84);
        let output: Vec<u8, 9> = to_vec(&response).unwrap();
        let back: Message = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Message::Response(n) if n == 84));
    }
}
