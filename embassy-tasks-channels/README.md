# RFB using embassy tasks and channels

Three embassy tasks happily sharing access to an embassy MPMC channel of message type `()`.
In the main loop, a `select` chooses between collecting `()`s from the channel and then incrementing a counter,
and receiving serial data.
