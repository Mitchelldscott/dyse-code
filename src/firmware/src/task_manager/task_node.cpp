/********************************************************************************
 * 
 *      ____                     ____          __           __       _          
 *	   / __ \__  __________     /  _/___  ____/ /_  _______/ /______(_)__  _____
 *	  / / / / / / / ___/ _ \    / // __ \/ __  / / / / ___/ __/ ___/ / _ \/ ___/
 *	 / /_/ / /_/ (__  )  __/  _/ // / / / /_/ / /_/ (__  ) /_/ /  / /  __(__  ) 
 *	/_____/\__, /____/\___/  /___/_/ /_/\__,_/\__,_/____/\__/_/  /_/\___/____/  
 *	      /____/                                                                
 * 
 * 
 * 
 ********************************************************************************/

#include "task_manager/task_node.h"

TaskNode::TaskNode(Task* p, int rate, int n_inputs, int* input_ids) {
	/*
		TaskNode constructor
		@param
			p: (Tasks*) The driving tasks for this node
			n_inputs: (int) number of input nodes
			input_ids: (int*) the identifiers of input nodes (in order of concatenation),
				does not specify the tasks index in the syncor list but a unique 
				tasks id associated with each tasks.
	*/
	timestamp = 0;
	set_task(p, rate);
	set_inputs(input_ids, n_inputs);
}

void TaskNode::latch(int value) {
	latch_flag = value;
}

bool TaskNode::is_latched() {
	return latch_flag;
}

int TaskNode::n_inputs() {
	/*
		Get the number of input nodes.
		@return
			size: (int) number of nodes to recieve input from.
	*/
	return input_ids.size();
}

int TaskNode::input_id(int index) {
	/*
		Get an id at index from the input id list
		@return
			id: (int) id of input at index
	*/
	return input_ids[index];
}

void TaskNode::reset_config() {
	/*
		Zero the current config to ensure reinitialization.

	*/
	task->reset();
	parameter_buffer.reset(0);
}

bool TaskNode::is_configured() {
	/*
		Check if the tasks is configured. Needed because
		setup data may be sent in chunks.
		@return
			status: (bool) if tasks is configured
	*/
	return (*task)[PARAM_DIMENSION] == parameter_buffer.size();
}

bool TaskNode::is_linked() {
	/*
		Check if the tasks is configured. Needed because
		setup data may be sent in chunks.
		@return
			status: (bool) if tasks is configured
	*/
	return inputs.size() == input_ids.size();
}

int TaskNode::n_links() {
	return inputs.size();
}

void TaskNode::set_inputs(int* inputids, int n_inputs) {
	/*
		Set the unique IDs of all input nodes.
		@param
			input_ids: (int*) unique IDs of input nodes.
			n_inputs: (int) number of input nodes.
	*/
	inputs.reset(0);
	input_ids.from_array(inputids, n_inputs);
}

void TaskNode::link_input(TaskNode* node, int idx) {
	/*
		Add a pointer to the node for faster lookup.
		Must pass inputs to this in the same order as,
		input_ids.
		@param
			node: (TaskNode*) pointer to an input node.
	*/
	if (idx == inputs.size()) {
		inputs.push(node);
	}
}

void TaskNode::set_task(Task* new_task, int rate) {
	/*
		Set the task that drives (input, config) -> (context, output).
		Also resets the task and output vector to ensure initial
		state is the zero state.
		@param
			task: (Task) User defined task.
	*/
	millis_rate = rate;
	task = new_task;
	input_buffer.reset((*task)[INPUT_DIMENSION]);
	output_buffer.reset((*task)[OUTPUT_DIMENSION]);
	reset_config();
}

void TaskNode::configure(int chunk_id, int chunk_size, float* data) {
	if (chunk_id * chunk_size == parameter_buffer.size()) {
		parameter_buffer.append(data, chunk_size);
	}
	else if (chunk_id * chunk_size < parameter_buffer.size()) {
		parameter_buffer.insert(data,
						chunk_id * chunk_size, 
						chunk_size);
	}
}

bool TaskNode::setup_task() {
	/*
		Call the tasks user defined setup function with the config buffer.
		Does nothing when not configured.
		@return
			status: (bool) if setup was called.
	*/
	if (is_configured()) {
		task->setup(&parameter_buffer);
		return true;		
	}
	return false;
}

void TaskNode::collect_inputs() {
	int curr_size = 0;
	for (int i = 0; i < inputs.size(); i++) {
		input_buffer.insert((*inputs[i])[OUTPUT_DIMENSION]->as_array(), curr_size, (*inputs[i])[OUTPUT_DIMENSION]->size());
		curr_size += (*inputs[i])[OUTPUT_DIMENSION]->size();
	}

}

bool TaskNode::run_task(float timer) {
	/*
		Call the tasks user defined run function. Does nothing if not configured
		or if inputs size is not equal to number of input ids.
		@param
			input_buffer: (Vector<float>*) concatenated outputs of tasks
				listed in input_ids (always the correct shape)
		@return
			status: (bool) if run was called.
	*/
	float duration = (timer - timestamp);
	// printf("Duration, %f %f\n", duration, timer);
	if (is_configured() && is_linked() && int(duration * 1000) >= millis_rate) {
		if (latch_flag == 0) {
			collect_inputs();
			task->run(&input_buffer, &output_buffer, duration);
		}
		else if (latch_flag == 2) {
			task->run(&input_buffer, &output_buffer, duration);
		}

		timestamp = timer;
		return true;
	}

	return false;
}

void TaskNode::print() {
	/*
		Dump all task info. Requires that the user defined a print
		for their task.
	*/
	task->print();
}

void TaskNode::print_output() {
	/*
		Dump output buffer.
	*/
	printf("\t");
	output_buffer.print();
}

void TaskNode::status(byte* buffer) {
	buffer[0] = (*task)[INPUT_DIMENSION];
	buffer[1] = (*task)[OUTPUT_DIMENSION];
	buffer[2] = (*task)[PARAM_DIMENSION];
	buffer[3] = inputs.size();
	buffer[4] = parameter_buffer.size();
	buffer[5] = is_latched();
}

// define the template specialization
// for fancy printing
template <> void Vector<TaskNode*>::print() {
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