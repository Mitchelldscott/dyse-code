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
#include "task_manager/task_node.h"

FTYK timers;

int main() {

	int errors = 0;
	unit_test_splash("Task Node", 0);

	int empty = 0;
	float tmp[1] = {0.6};
	int cmf_inputs[2] = {1, 0};

	printf("=== Init task Factory ===\n");

	printf("=== Init Node List ===\n");
	TaskNode* nodelist[2];

	printf("=== Add LSM6DSOX ===\n");
	timers.set(0);
	nodelist[0] = new TaskNode(new_task("LSM"), 0, &empty);
	errors += assert_leq<float>(timers.micros(0), 14000, "LSM new node timer (us)");

	printf("=== Add Complimentary Filter ===\n");
	timers.set(0);
	nodelist[1] = new TaskNode(new_task("CMF"), 2, cmf_inputs);
	errors += assert_leq<float>(timers.micros(0), 5, "CMF new node timer (us)");

	printf("=== Run task 0 ===\n");
	nodelist[0]->run_task();
	nodelist[0]->run_task();
	nodelist[0]->run_task();
	nodelist[0]->print();
	nodelist[0]->print_output();

	printf("=== Run Setup/Link task 1 ===\n");
	timers.delay_millis(0, 1);
	bool status = nodelist[1]->run_task();		// try to run the task without calling setup or linking nodes
	errors += assert_eq<int>(int(status), 0, "No setup/links run: status check");
	
	timers.delay_millis(0, 1);

	Vector<float>* config = (*nodelist[1])[PARAM_DIMENSION];		// Get the config buffer
	config->from_array(tmp, 1);							// set the config, make sure if the config gets filled setup is called (config.size() is how configuration is checked)
	nodelist[1]->setup_task();							// call setup to initialize the task	
	status = nodelist[1]->run_task();					// Run the task with setup
	errors += assert_eq<int>(int(status), 0, "No links run: status check");

	timers.delay_millis(0, 1);
	nodelist[1]->link_input(nodelist[0], 0);	// link task0
	nodelist[1]->link_input(nodelist[1], 1);	// link task1
	status = nodelist[1]->run_task();		// run
	errors += assert_eq<int>(int(status), 1, "Linked and setup run: status check");
	
	nodelist[1]->print();
	nodelist[1]->print_output();

	printf("=== Reconfigure and Setup task 1 ===\n");
	nodelist[1]->reset_config();						// reset the config

	status = nodelist[1]->run_task();					// try to run the task after reset without calling setup
	errors += assert_eq<int>(int(status), 0, "No setup run: status check");

	config = (*nodelist[1])[PARAM_DIMENSION];			// Get the config buffer
	config->from_array(tmp, 1);							// set the config, make sure if the config gets filled setup is called (config.size() is how configuration is checked)
	nodelist[1]->setup_task();							// call setup to initialize the task

	status = nodelist[1]->run_task();					// Run the task with setup
	
	errors += assert_eq<int>(int(status), 1, "Reset, Reconfigure & run: status check");
	nodelist[1]->print();
	nodelist[1]->print_output();

	printf("=== Finished Graph Node tests with %i errors ===\n", errors);
}