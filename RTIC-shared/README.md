# RFB using RTIC and a shared RTIC variable

Three RTIC tasks for digital inputs share access to a shared `RTIC` variable with the `idle` task,
which handles serial port communication.

The digital input tasks have `priority = 1`.

Will this be comparable to using an atomic, or perhaps better?