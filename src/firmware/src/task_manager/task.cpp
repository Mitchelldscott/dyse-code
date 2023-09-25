#include "task_manager/task.h"

Task::Task() {
	dimensions.resize(TASK_DIMENSIONS);
	dimensions.set_items(TASK_DIMENSIONS);
}

void Task::reset() {
	/*
		Base implementation of Task functions.
		Defined to show user when inheritance has issues
		or a required Task function is not implemented.
	*/
	printf("Task Base Object: Reset\n");
}

void Task::clear() {
	/*
		Base implementation of Task functions.
		Defined to show user when inheritance has issues
		or a required Task function is not implemented.
	*/
	printf("Task Base Object: Clear\n");
}

void Task::print() {
	/*
		Base implementation of Task functions.
		Defined to show user when inheritance has issues
		or a required Task function is not implemented.
	*/
	printf("Task Base Object: Print\n");
}

// int Task::input_dim() {
// 	/*
// 		Base implementation of Task functions.
// 		returns dimensions of the input.
// 		@return
// 			dimension: (int) size of input
// 	*/
// 	return dimensions[0];
// }

// int Task::params_dim() {
// 	/*
// 		Base implementation of Task functions.
// 		returns dimensions of the output.
// 		@return
// 			dimension: (int) size of input
// 	*/
// 	return dimensions[3];
// }

// int Task::output_dim() {
// 	/*
// 		Base implementation of Task functions.
// 		returns dimensions of the output.
// 		@return
// 			dimension: (int) size of input
// 	*/
// 	return dimensions[2];
// }

void Task::setup(Vector<float>* config) {
	/*
		Base implementation of Task functions.
		Defined to show user when inheritance has issues
		or a required Task function is not implemented.
		@param
			config: (Vector<float>) Vector of configuration data, 
				organization is handled by user
	*/
	printf("Task Base Object: Setup\n");
	config->print();
}

void Task::run(Vector<float>* input, Vector<float>* output, float dt) {
	/*
		Base implementation of Task functions.
		Defined to show user when inheritance has issues
		or a required Task function is not implemented.
		@param
			input: (Vector<float>*) flattened Vector of input data for Task
			output: (Vector<float>*) flattened Vector of output data from Task
	*/
	printf("Task Base Object: Run\n");
	output->reset(0);
}

// define the template specialization
// for fancy printing
template <> void Vector<Task*>::print() {
	if (length == 0) {
		printf("Task Vector [empty]\n");
		return;
	}

	printf("Task Vector [%i]: [\n", length);
	for (int i = 0; i < length; i++) {
		buffer[i]->print();
	}
	printf("]\n");
}





