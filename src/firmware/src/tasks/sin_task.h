#ifndef SIN_DRIVER_H
#define SIN_DRIVER_H

#include "task_manager/task.h"


class SinTask: public Task {
	private:
		char key[3] = {'S', 'I', 'N'};
		float frequency;
		float amplitude;
		float shift;
		float counter;


	public:
		SinTask() {
			dimensions.reset(TASK_DIMENSIONS);
			dimensions[INPUT_DIMENSION] = 0;
			dimensions[PARAM_DIMENSION] = 3;
			dimensions[OUTPUT_DIMENSION] = 1;

			counter = 0;
			
			reset();
		}

		void setup(Vector<float>* config) {
			frequency = (*config)[0];
			amplitude = (*config)[1];
			shift = (*config)[2];
			// print();
			// analogWriteResolution(12);
		}

		void reset() {
			frequency = 0;
			amplitude = 0;
			shift = 0;
		}

		void clear() {
			frequency = 0;
			amplitude = 0;
			shift = 0;
		}

		void run(Vector<float>* inputs, Vector<float>* outputs, float dt) {
			// printf("dt: %f + %f\n", counter, dt);
			counter += dt;
			(*outputs)[0] = (amplitude * sin(frequency * counter)) + shift;
		}

		void print() {
			Serial.println("Constant Sin task");
			Serial.printf("\tFrequency: %i\n", frequency);
			Serial.printf("\tAmplitude: %i\n", amplitude);
		}
};

#endif