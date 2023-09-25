#ifndef PWM_DRIVER_H
#define PWM_DRIVER_H

#include "utilities/timing.h"
#include "task_manager/task.h"

#define WRITE_RESOLUTION 12
#define RESOLUTION_SCALE 4096

class PwmDriver: public Task {
	private:
		char key[3] = {'P', 'W', 'M'};
		int pin;
		int output;


	public:
		PwmDriver() {
			dimensions.reset(TASK_DIMENSIONS);
			dimensions[INPUT_DIMENSION] = 1;
			dimensions[PARAM_DIMENSION] = 1;
			dimensions[OUTPUT_DIMENSION] = 1;

			reset();
		}

		void setup(Vector<float>* config) {
			pin = (*config)[0];
			pinMode(pin, OUTPUT);
			analogWriteResolution(WRITE_RESOLUTION);
		}

		void reset() {
			output = 0;
		}

		void clear() {
			output = 0;
		}

		void run(Vector<float>* inputs, Vector<float>* outputs, float dt) {
			output = int(max(0.0, min(1.0, (*inputs)[0])) * RESOLUTION_SCALE);
			analogWrite(pin, output);
			(*outputs)[0] = float(output);
		}

		void print() {
			Serial.println("PWM Driver");
			Serial.printf("\tPin: %i\n", pin);
			Serial.printf("\tOutput: %i\n", output);
		}
};

#endif