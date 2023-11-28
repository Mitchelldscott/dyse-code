## Build firmware
        make

## Build and upload firmware (need to be connected by USB to Teensy)
        make upload 
        
## Firmware status guide (terminology)

Onboard-LED blinking = HID connected

Red-LED blinking = configuration issue

Green-LED on = tasks are running

| Status       | Teensy Powered | Teensy configured | Motors Powered | HID Connected |
| ------------ | -------------- | ----------------- | -------------- | ------------- |
| **Dead/off** | Yes/No         | No                | No             | No            |
| **Active**   | Yes            | No                | Yes            | No            |
| **Alive**    | Yes            | Yes               | Yes            | Yes/No        |

# Pipeline Plumbing TODO's
