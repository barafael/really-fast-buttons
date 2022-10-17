#![cfg_attr(not(test), no_std)]

use serde::{Deserialize, Serialize};

pub use heapless::Vec;
pub use postcard::from_bytes;
pub use postcard::to_vec;

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
    fn request() {
        let request = Message::Request;
        let output: Vec<u8, 9> = to_vec(&request).unwrap();
        let back: Message = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Message::Request));
    }

    #[test]
    fn response() {
        let response = Message::Response(84);
        let output: Vec<u8, 9> = to_vec(&response).unwrap();
        let back: Message = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Message::Response(n) if n == 84));
    }
}
