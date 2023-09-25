#ifndef CONST_DRIVER_H
#define CONST_DRIVER_H

#include "task_manager/task.h"


class ConstTask: public Task {
	private:
		char key[3] = {'V', 'A', 'L'};
		int value;


	public:
		ConstTask() {
			dimensions.reset(TASK_DIMENSIONS);
			dimensions[INPUT_DIMENSION] = 0;
			dimensions[PARAM_DIMENSION] = 1;
			dimensions[OUTPUT_DIMENSION] = 1;

			reset();
		}

		void setup(Vector<float>* config) {
			value = (*config)[0];
			// print();
			// analogWriteResolution(12);
		}

		void reset() {
			value = 0;
		}

		void clear() {
			value = 0;
		}

		void run(Vector<float>* inputs, Vector<float>* outputs, float dt) {
			(*outputs)[0] = float(value);
		}

		void print() {
			Serial.println("Constant Value task");
			Serial.printf("\tOutput: %i\n", value);
		}
};

// class ConstTask: public Task {
// 	private:
// 		char key[3] = {'N', 'C', 'V'};
// 		int n;
// 		float* buffer; 


// 	public:
// 		ConstTask() {
// 			dimensions.reset(TASK_DIMENSIONS);
// 			dimensions[INPUT_DIMENSION] = 0;
// 			dimensions[PARAM_DIMENSION] = 0;
// 			dimensions[OUTPUT_DIMENSION] = 0;

// 			reset();
// 		}

// 		void setup(Vector<float>* config) {
// 			n = config->size();
// 			buffer = config->as_array();
// 			dimensions[PARAM_DIMENSION] = n;
// 			dimensions[OUTPUT_DIMENSION] = n;
// 		}

// 		void reset() {
// 			value = 0;
// 		}

// 		void clear() {
// 			value = 0;
// 		}

// 		void run(Vector<float>* inputs, Vector<float>* outputs) {
			
// 			(*outputs)[0] = float(value);
// 		}

// 		void print() {
// 			Serial.println("Constant Value task");
// 			Serial.printf("\tOutput: %i\n", value);
// 		}
// };

#endif