# RFB using RTIC and an atomic variable

Three RTIC tasks for digital inputs share access to an atomic with the `idle` task,
which handles serial port communication.

The digital input tasks have `priority = 1`.
