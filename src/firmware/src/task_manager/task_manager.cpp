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

#include "task_manager/task_manager.h"

FTYK sys_timers;
float sys_lifetime = 0;

int run_status = 0;
int config_status = 0;

Vector<int> node_ids(0);
Vector<TaskNode*> nodes(0);
CommsPipeline* pipeline_internal;

CommsPipeline* init_task_manager() {
	sys_timers.set(0);		// setup master cycle timer
	sys_timers.set(1);		// setup individual run timer
	sys_timers.set(2);		// status led control rate

	pinMode(RUN_STATUS_PIN, OUTPUT);
	pinMode(CONFIGURATION_STATUS_PIN, OUTPUT);

	pipeline_internal = enable_hid_interrupts();
	return pipeline_internal;
}

int node_index(int id) {
	return node_ids.find(id);
}

bool link_nodes(int index) {
	for (int i = nodes[index]->n_links(); i < nodes[index]->n_inputs(); i++) {
		int node_idx = node_index(nodes[index]->input_id(i));
		if (node_idx >= 0) {
			printf("Linking node %i %i %i\n", index, node_idx, i);
			nodes[index]->link_input(nodes[node_idx], i);
		}
		else {
			return false;
		}
	}
	return true;
}

void add_task(TaskSetupPacket* task_init) {
	int index = node_index(task_init->task_id);
	printf("Adding %i %i %c%c%c\n", task_init->task_id, task_init->rate, task_init->key[0], task_init->key[1], task_init->key[2]);

	if (index == -1) {	// Node does not exist yet
		// Add new node, node id
		nodes.push(new TaskNode(new_task(task_init->key), task_init->rate, task_init->n_inputs, task_init->inputs.as_array()));
		node_ids.push(task_init->task_id);
		index = nodes.size() - 1;

		TaskFeedback* task_fb = new TaskFeedback;
		task_fb->latch = 0;
		task_fb->configured = 0;
		task_fb->task_id = index;
		task_fb->output.reset(0);
		task_fb->timestamp = -1;

		noInterrupts();
		pipeline_internal->feedback.push(task_fb);
		interrupts();
	}
	else { // Node exists so update it's params (and deconfig)
		noInterrupts();
		pipeline_internal->feedback[index]->configured = 0;
		interrupts();

		nodes[index]->set_task(new_task(task_init->key), task_init->rate);
		nodes[index]->set_inputs(task_init->inputs.as_array(), task_init->n_inputs);
	}

	nodes[index]->latch(0); // always unlatch here
}

void update_task(TaskSetupPacket* task_params) {
	int node_idx = node_index(task_params->task_id);

	if (node_idx == -1) {
		printf("Node %i not found for update\tThis should never happen\n", task_params->task_id);
		nodes.print();
		return;
	}
	if (nodes[node_idx]->is_configured()) {
		printf("Node %i is already configured\tThis should never happen\n", task_params->task_id);
		return;
	}

	nodes[node_idx]->configure(task_params->chunk_id * task_params->chunk_size, 
								task_params->chunk_size,
								task_params->parameters.as_array());


	int config = nodes[node_idx]->setup_task();
	printf("Configure node %i %i %i %i\n", node_idx, task_params->chunk_id * task_params->chunk_size, task_params->chunk_size, config);

	if (!config) {
		printf("failed config: %i\n", (*nodes[node_idx])[PARAM_DIMENSION]->size());
	}

	noInterrupts();
	pipeline_internal->feedback[node_idx]->configured = config;
	interrupts();
}

void overwrite_task(TaskSetupPacket* task_update) {
	int node_idx = node_index(task_update->task_id);
	if (node_idx == -1) {
		printf("Node %i not found for overwrite\tThis should never happen\n", task_update->task_id);
		return;
	}
	if (!nodes[node_idx]->is_configured()) {
		printf("Node %i not configured for overwrite\tThis should never happen\n", task_update->task_id);
		return;
	}

	// printf("Overwrite node %i %i %i\n", node_idx, task_update->task_id, task_update->data_len);
	// need to send another overwrite to unlatch
	if (task_update->latch == 1 && (*nodes[node_idx])[OUTPUT_DIMENSION]->size() == task_update->data.size()) {
		*(*nodes[node_idx])[OUTPUT_DIMENSION] = task_update->data;
		nodes[node_idx]->latch(task_update->latch);
	}
	else if (task_update->latch == 2 && (*nodes[node_idx])[INPUT_DIMENSION]->size() == task_update->data.size()) {
		*(*nodes[node_idx])[INPUT_DIMENSION] = task_update->data;
		nodes[node_idx]->latch(task_update->latch);
	}
	else if (task_update->latch == 0) {
		nodes[node_idx]->latch(0);
	}
}

void task_setup_handler() {
	noInterrupts();
	int n_items = pipeline_internal->setup_queue.size();
	interrupts();

	for (int i = 0; i < n_items; i++) {
		noInterrupts();
		TaskSetupPacket* p = pipeline_internal->setup_queue.pop();
		interrupts();
		// printf("Pop: %i %i %p\n", i, n_items, p);

		if (p != NULL) {
			switch (p->packet_type) {
				case 0:
					// printf("Init node: %i\n", p->task_id);
					add_task(p);
					break;

				case 1:
					// printf("Config node: %i\n", p->task_id);
					update_task(p);
					break;

				case 2:
					// printf("Latch node: %i\n", p->task_id);
					overwrite_task(p);
					break;

				default:
					// printf("Default Packet type: this should never happen %i\n", p->packet_type);
					break;
			}
			
			delete p;
		}
	}
}

void task_publish_handler(int i) {
	// TaskFeedback* task_fb = new TaskFeedback;
	// task_fb->task_id = node_ids[i];
	// task_fb->latch = nodes[i]->is_latched();
	// task_fb->timestamp = sys_timers.millis(1);
	// task_fb->output.from_array((*nodes[i])[OUTPUT_DIMENSION]->as_array(), (*nodes[i])[OUTPUT_DIMENSION]->size());

	noInterrupts();
	pipeline_internal->feedback[i]->update += 1;
	pipeline_internal->feedback[i]->latch = nodes[i]->is_latched();
	pipeline_internal->feedback[i]->timestamp = sys_timers.millis(1);
	pipeline_internal->feedback[i]->output.from_array((*nodes[i])[OUTPUT_DIMENSION]->as_array(), (*nodes[i])[OUTPUT_DIMENSION]->size());
	interrupts();
}

void spin() {
	// handle queued setup
	task_setup_handler();

	// handle task execution
	for (int i = 0; i < nodes.size(); i++) {
		// If task isn't fully linked to inputs this will link them (if the input tasks exist)
		if (link_nodes(i)) {
			// printf("Linked: %i\tconfigured %i\tinputs %i\tlinks %i\n", i, nodes[i]->is_configured(), nodes[i]->n_inputs(), nodes[i]->n_links());
			// pulls outputs from input tasks and runs the current task
			sys_timers.set(1);
			if (nodes[i]->run_task(sys_timers.total_seconds(0))){
				// printf("Ran: %i %i %f\n", i, nodes[i]->is_latched(), sys_timers.total_seconds(0));
				// put task output, context in the pipeline for publishing
				task_publish_handler(i);
				run_status = 1;
			}
		}
		else {
			printf("Linking error %i\n", i);
		}

		noInterrupts();
		pipeline_internal->lifetime = sys_timers.total_seconds(0); // update lifetime everytime a task is run
		interrupts();
	}

	if (sys_timers.secs(2) > 0.25) {
		sys_timers.set(2);

		if (nodes.size() == 0) {
			config_status = !config_status;
		}
		else {
			config_status = 0;
			for (int i = 0; i < nodes.size(); i++) {
				config_status |= !(nodes[i]->is_configured() && nodes[i]->is_linked());
			}
		}

		digitalWriteFast(RUN_STATUS_PIN, run_status);
		digitalWriteFast(CONFIGURATION_STATUS_PIN, config_status);
	}
}

void dump_all_tasks() {
	printf("Task Manager lifetime: %f\n", sys_lifetime);
	printf("Node ids\n\t");
	node_ids.print();
	printf("Nodes\n\t");
	nodes.print();
}
