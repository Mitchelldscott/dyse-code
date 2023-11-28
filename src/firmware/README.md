## Build firmware
        make

## Build and upload firmware (need to be connected by USB to Teensy)

        make
## Firmware status guide (terminology)

LED blinking = HID connected

| Status       | Teensy Powered | Teensy configured | Motors Powered | HID Connected |
| ------------ | -------------- | ----------------- | -------------- | ------------- |
| **Dead/off** | Yes/No         | No                | No             | No            |
| **Active**   | Yes            | No                | Yes            | No            |
| **Alive**    | Yes            | Yes               | Yes            | Yes/No        |

# Pipeline Plumbing TODO's

### DR16 example
 - update to print and show safety switch value in serial console (use tycmd)

### Estimators
