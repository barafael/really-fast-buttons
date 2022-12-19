# RFB using futures and a `select!` with embassy

This example selects between three futures generated with `paX.wait_for_rising_edge()`.
This select is polled within a loop, and when one future wins, an atomic is incremented.
The futures are dropped and re-generated each loop iteration, leading to a low reactivity.

I really like the conciseness of this solution, but it is not correct.