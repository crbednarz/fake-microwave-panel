# Microwave Electronics for Ikea DUKTIG

This is a small project to add a semi-functional control panel to an Ikea DUKTIG.

Features:

- Buttons for start, stop, and time input
- Speaker for typical microwave beeps and running noise
- 7-segment display for time display
- IR emitter for controller light inside the microwave
- Magnet sensor for door open detection


## Video

https://github.com/user-attachments/assets/23268392-00d4-439c-b72a-91c00f4df706

## Parts

Many of these parts I already had laying around, so there's likely a more cost effective solution out there.

- [Firebeetle ESP32](https://www.digikey.com/en/products/detail/dfrobot/DFR0654/13978504) - Low power MCU capable of audio output
- [IR LED](https://www.digikey.com/en/products/detail/everlight-electronics-co-ltd/IR333-A/2675571) - For turning the in-microwave light on/off.
- [LED light puck](https://www.amazon.com/dp/B09MQMT2WK?ref_=ppx_hzsearch_conn_dt_b_fed_asin_title_16&th=1) - For lighting the inside of the microwave.
- [Keypad](https://www.amazon.com/dp/B07RY85MGF) - For time input
- [Magnetic switch](https://www.amazon.com/dp/B085XQLQ3N) and [Magnets](https://www.amazon.com/dp/B072K5SLXK) - For door opening detection
- [7-Segment Display](https://www.amazon.com/dp/B07MCGDST2) - For time display
- [Speaker](https://www.amazon.com/dp/B01LN8ONG4) - For beeps and running sound
- 2x Buttons - For start/stop
- Various wires, resistors, breadboard, etc.
