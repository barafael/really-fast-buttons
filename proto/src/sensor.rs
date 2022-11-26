use postcard::fixint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, defmt::Format)]
#[repr(C)]
pub enum Request {
    GetCount,
    WhoAreYou,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, defmt::Format)]
#[repr(C)]
pub enum Response<'a> {
    #[serde(with = "fixint::le")]
    Count(u32),
    IAm(&'a str),
}

#[cfg(all(test, feature = "sensor"))]
mod test {
    use core::ops::Deref;
    use heapless::Vec;
    use postcard::{from_bytes, to_vec};

    use super::*;

    #[test]
    fn request() {
        let request = Request::GetCount;
        let output: Vec<u8, 1> = to_vec(&request).unwrap();
        let back: Request = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Request::GetCount));
    }

    #[test]
    fn max_count() {
        let response = Response::Count(u32::MAX);
        let output: Vec<u8, 5> = to_vec(&response).unwrap();
        let back: Response = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Response::Count(n) if n == u32::MAX));
    }

    #[test]
    fn min_count() {
        let response = Response::Count(u32::MIN);
        let output: Vec<u8, 5> = to_vec(&response).unwrap();
        let back: Response = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Response::Count(n) if n == u32::MIN));
    }

    #[test]
    fn is() {
        let response = Response::IAm("or is it?");
        let output: Vec<u8, 11> = to_vec(&response).unwrap();
        let back: Response = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Response::IAm(s) if s == "or is it?"));
    }
}
