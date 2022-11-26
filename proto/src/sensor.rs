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
    fn response() {
        let response = Response::Count(84);
        let output: Vec<u8, 9> = to_vec(&response).unwrap();
        let back: Response = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Response::Count(n) if n == 84));
    }

    #[test]
    fn is() {
        let response = Response::IAm("or is it?");
        let output: Vec<u8, 11> = to_vec(&response).unwrap();
        let back: Response = from_bytes(output.deref()).unwrap();
        assert!(matches!(back, Response::IAm(s) if s == "or is it?"));
    }
}