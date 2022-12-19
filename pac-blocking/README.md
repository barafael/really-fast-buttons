# RFB-pac-blocking

Polls PA0, PA1, PA2, incrementing a global atomic,
which is read out in the serial RXNE interrupt.
The peripheral access crate is used here, for the monitoring itself.

Edge detection is done by software.

Ironically, includes an interrupt-driven serial port implementation.
Otherwise I'd have to poll the serial registers as well, defeating the purposee.
This highlights the problem with the blocking approach obviously.
