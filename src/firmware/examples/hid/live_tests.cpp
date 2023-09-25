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

#include "utilities/timing.h"
#include "utilities/splash.h"
#include "utilities/assertions.h"
#include "hid_comms/hid_comms.h"

#define MASTER_CYCLE_TIME_MS 	0.5
#define MASTER_CYCLE_TIME_S 	(MASTER_CYCLE_TIME_MS * 1E-3)
#define MASTER_CYCLE_TIME_US 	(MASTER_CYCLE_TIME_MS * 1E3 )
#define MASTER_CYCLE_TIME_ERR 	(MASTER_CYCLE_TIME_MS + 0.01)

#define TEST_DURATION_S			30

FTYK timers;
int tasks = 0;

void dump_feedback_packet(CommsPipeline* comms_pipe, int id) {
	// TaskFeedback* task_fb = new TaskFeedback;
	// task_fb->latch = 0;
	// task_fb->task_id = id;
	// task_fb->output.reset(5);
	// task_fb->output.set_items(5);
	// task_fb->timestamp = -1;
	// printf("Dumping %i\n", id);

	noInterrupts();
	comms_pipe->feedback[id]->output[4] = 1.0;
	interrupts();
}

void add_feedback_packet(CommsPipeline* comms_pipe, int id) {
	TaskFeedback* task_fb = new TaskFeedback;
	task_fb->latch = 0;
	task_fb->task_id = id;
	task_fb->output.reset(5);
	task_fb->timestamp = -1;
	// printf("Dumping %i\n", id);

	noInterrupts();
	comms_pipe->feedback.push(task_fb);
	interrupts();
}

void task_setup_handler(CommsPipeline* comms_pipe) {
	// consume all input packets
	noInterrupts();
	int n_items = comms_pipe->setup_queue.size();
	interrupts();

	if (n_items <= 0) {
		return;
	}

	for (int i = 0; i < n_items; i++) {
		noInterrupts();
		TaskSetupPacket* p = comms_pipe->setup_queue.pop();
		interrupts();

		if (p) {
			printf("Consuming item %i of %i, %i\n", i+1, n_items, p->packet_type);
			if (p->packet_type == 0) {
				add_feedback_packet(comms_pipe, tasks);
				tasks += 1;
			}
			
			delete p;
		}
	}
}

// Runs once
int main() {

	int errors = 0;

	unit_test_splash("Live comms", TEST_DURATION_S);

	CommsPipeline* comms_pipe = enable_hid_interrupts();

	while (!usb_rawhid_available()) {};

	int lifetime = 0;
	timers.set(0);
	timers.set(1);
	while (lifetime < TEST_DURATION_S) {

		task_setup_handler(comms_pipe);
		for(int i = 0; i < tasks; i++) {
			noInterrupts();
			comms_pipe->lifetime += timers.secs(1);
			timers.set(1);
			interrupts();

			dump_feedback_packet(comms_pipe, i);
						
		}

		float cycletime = timers.delay_millis(0, MASTER_CYCLE_TIME_MS);
		timers.set(0);
		lifetime += MS_2_S(cycletime);
		errors += assert_leq<float>(cycletime, MASTER_CYCLE_TIME_ERR, "Teensy overcycled (ms)");
	}

	printf(" * Finished tests in %fs\n", comms_pipe->lifetime);
	printf(" * %i failed\n", int(errors + hid_errors));

	while(1) {};
	return 0;

}
