#include "utilities/splash.h"
#include "utilities/assertions.h"
#include "task_manager/task_manager.h"

#define MASTER_CYCLE_TIME_US 	1000.0
#define MASTER_CYCLE_TIME_MS 	1.0
#define MASTER_CYCLE_TIME_S 	0.001
#define MASTER_CYCLE_TIME_ERR 	1.001 // ms


FTYK timers;
CommsPipeline* comms_pipe;

int main() {
	int errors = 0;

	unit_test_splash("Task Manager", -1);
	
	int cmf_inputs[2] = {9, 10};

	TaskSetupPacket* lsm = new TaskSetupPacket;
	lsm->key = "LSM";
	lsm->task_id = 10;
	lsm->n_inputs = 0;
	lsm->packet_type = 0;

	TaskSetupPacket* cmf = new TaskSetupPacket;
	cmf->key = "CMF";
	cmf->task_id = 9;
	cmf->n_inputs = 2;
	cmf->inputs.from_array(cmf_inputs, 2);
	cmf->packet_type = 0;

	TaskSetupPacket* cmf_params = new TaskSetupPacket;
	cmf_params->task_id = 9;
	cmf_params->chunk_id = 0;
	cmf_params->chunk_size = 1;
	cmf_params->parameters.reset(1);
	cmf_params->parameters[0] = 0.6;
	cmf_params->packet_type = 1;

	printf("\n=== Add nodes and parameters manually ===\n");

	timers.set(0);
	comms_pipe = init_task_manager();
	errors += assert_leq<float>(timers.micros(0), 2, "TM init timer");
	
	timers.set(1);
	add_task(lsm);
	errors += assert_leq<float>(timers.millis(1), 15, "LSM add timer");

	timers.set(1);
	add_task(cmf);
	errors += assert_leq<float>(timers.micros(1), 7, "CMF add timer");

	timers.set(1);
	update_task(cmf_params);
	errors += assert_leq<float>(timers.micros(1), 2, "CMF update timer");

	for (int i = 0; i < 3; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}

	dump_all_tasks();

	printf("\n=== Reset nodes and update parameters from the comms_pipeline ===\n");

	// reset the tasks so they require a reconfig
	comms_pipe->setup_queue.push(lsm);
	comms_pipe->setup_queue.push(cmf);
	comms_pipe->setup_queue.push(cmf_params);

	dump_all_tasks();

	for (int i = 0; i < 5; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}
	
	dump_all_tasks();

	TaskSetupPacket* output_lock = new TaskSetupPacket;
	output_lock->packet_type = 2;
	output_lock->task_id = 10;
	output_lock->latch = 1;
	output_lock->data_len = 9;
	output_lock->data.reset(9);

	for (int i = 0; i < 9; i++) {
		output_lock->data[i] = 9;
	}

	printf("\n=== Passing lock through the pipeline 1 ===\n");

	comms_pipe->setup_queue.push(output_lock);

	for (int i = 0; i < 5; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}
	
	dump_all_tasks();


	printf("\n=== Reconfigure Tasks through the pipeline 1 ===\n");

	lsm = new TaskSetupPacket;
	lsm->key = "LSM";
	lsm->task_id = 10;
	lsm->n_inputs = 0;
	lsm->packet_type = 0;

	cmf = new TaskSetupPacket;
	cmf->key = "CMF";
	cmf->task_id = 9;
	cmf->n_inputs = 2;
	cmf->inputs.from_array(cmf_inputs, 2);
	cmf->packet_type = 0;

	cmf_params = new TaskSetupPacket;
	cmf_params->task_id = 9;
	cmf_params->chunk_id = 0;
	cmf_params->chunk_size = 1;
	cmf_params->parameters.reset(1);
	cmf_params->parameters[0] = 0.6;
	cmf_params->packet_type = 1;

	comms_pipe->setup_queue.push(lsm);
	comms_pipe->setup_queue.push(cmf);
	comms_pipe->setup_queue.push(cmf_params);

	for (int i = 0; i < 5; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}
	
	dump_all_tasks();

	printf("\n=== Passing lock through the pipeline 2 ===\n");

	output_lock = new TaskSetupPacket;
	output_lock->packet_type = 2;
	output_lock->task_id = 10;
	output_lock->latch = 1;
	output_lock->data_len = 9;
	output_lock->data.reset(9);

	comms_pipe->setup_queue.push(output_lock);

	for (int i = 0; i < 5; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}
	
	dump_all_tasks();


	printf("\n=== Reconfigure Tasks through the pipeline 2 ===\n");

	lsm = new TaskSetupPacket;
	lsm->key = "LSM";
	lsm->task_id = 10;
	lsm->n_inputs = 0;
	lsm->packet_type = 0;

	cmf = new TaskSetupPacket;
	cmf->key = "CMF";
	cmf->task_id = 9;
	cmf->n_inputs = 2;
	cmf->inputs.from_array(cmf_inputs, 2);
	cmf->packet_type = 0;

	cmf_params = new TaskSetupPacket;
	cmf_params->task_id = 9;
	cmf_params->chunk_id = 0;
	cmf_params->chunk_size = 1;
	cmf_params->parameters.reset(1);
	cmf_params->parameters[0] = 0.6;
	cmf_params->packet_type = 1;

	comms_pipe->setup_queue.push(lsm);
	comms_pipe->setup_queue.push(cmf);
	comms_pipe->setup_queue.push(cmf_params);

	for (int i = 0; i < 5; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}
	
	dump_all_tasks();

	printf("\n=== Passing lock through the pipeline 3 ===\n");

	output_lock = new TaskSetupPacket;
	output_lock->packet_type = 2;
	output_lock->task_id = 10;
	output_lock->latch = 1;
	output_lock->data_len = 9;
	output_lock->data.reset(9);

	comms_pipe->setup_queue.push(output_lock);

	for (int i = 0; i < 5; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}
	
	dump_all_tasks();



	printf("\n=== Reconfigure Tasks through the pipeline 3 ===\n");

	lsm = new TaskSetupPacket;
	lsm->key = "LSM";
	lsm->task_id = 10;
	lsm->n_inputs = 0;
	lsm->packet_type = 0;

	cmf = new TaskSetupPacket;
	cmf->key = "CMF";
	cmf->task_id = 9;
	cmf->n_inputs = 2;
	cmf->inputs.from_array(cmf_inputs, 2);
	cmf->packet_type = 0;

	cmf_params = new TaskSetupPacket;
	cmf_params->task_id = 9;
	cmf_params->chunk_id = 0;
	cmf_params->chunk_size = 1;
	cmf_params->parameters.reset(1);
	cmf_params->parameters[0] = 0.6;
	cmf_params->packet_type = 1;

	comms_pipe->setup_queue.push(lsm);
	comms_pipe->setup_queue.push(cmf);
	comms_pipe->setup_queue.push(cmf_params);

	for (int i = 0; i < 5; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}
	
	dump_all_tasks();

	printf("\n=== Passing lock through the pipeline 4 ===\n");

	output_lock = new TaskSetupPacket;
	output_lock->packet_type = 2;
	output_lock->task_id = 10;
	output_lock->latch = 1;
	output_lock->data_len = 9;
	output_lock->data.reset(9);

	comms_pipe->setup_queue.push(output_lock);

	for (int i = 0; i < 5; i++) {
		timers.set(1);
		spin();
		errors += assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Spin duration");
	}
	
	dump_all_tasks();


	printf("=== Finished Task Manager tests with %i errors ===\n", errors);
}