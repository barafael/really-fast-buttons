# Really Fast Button Project Protocol

Super simple... A request, and a destructive Response.
After a request is sent, a response with the current edge count shall be emitted.
After the response has been sent, the sensor-internal edge count shall be reset to 0.

## Feature: Actuator

Simple message type for requesting an actuator to generate a signal with a period of `period_picos`.
Duty cycle should be 50%.
It should generate `rising_edges` rising edges, then go back to low.
