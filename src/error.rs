use crate::Message;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to write item {cfg:?}"))]
    Msg { cfg: Message },
}
