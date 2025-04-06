This is a silly exploratory set of programs and a service I wrote to play around with embedded rust, async rust, a geiger counter, a raspberry pi, stochastic processes.
This is a embedded rust service for a raspberry pi 3/4/5 (could work on the others with tweeks), that reads geiger counter pulses as an input on a gpio pin.
This was designed for a mighty ohm geiger counter that has a pulse pin but so long as you can get an electrical pulse from any geiger counter this will work off of that just fine provided things are mapped right.

The service publishes the gieger counter state at the root address for port 1986 on the raspberry pi.
The endpoint will return a json like  following : 

{"last_count":"2025-04-06T08:21:37.586544901Z","count":6,"last_reading_cpm":15.999650962014385}

this includes the current count of the counter, the last count timestamp (tuned for a mighty ohm geiger counter by subtracting 100us) and the last reading in cpm

The readings are measured in intervals of 1 minute with a poll of 60 times per sec.
These values can be adjusted and in a later update may be args.

Additionally there are python scripts that are bundled with this application to monitor the geiger counter or roll dice with the geiger counter.

