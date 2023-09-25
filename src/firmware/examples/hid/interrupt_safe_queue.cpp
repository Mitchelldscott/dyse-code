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

#define MASTER_CYCLE_TIME_MS 	500
#define MASTER_CYCLE_TIME_S 	(MASTER_CYCLE_TIME_MS * 1E-3)
#define MASTER_CYCLE_TIME_US 	(MASTER_CYCLE_TIME_MS * 1E3 )
#define MASTER_CYCLE_TIME_ERR 	(MASTER_CYCLE_TIME_MS + 0.01)

#define TEST_DURATION_S			30

FTYK timers;


struct DummyObject {
	Vector<float> arr1;
	Vector<float> arr2;
}

Vector<DummyObject*> pipeline;

// Runs once
int main() {

	int errors = 0;
	float lifetime = 0;

	unit_test_splash("Interrupt safe queue", TEST_DURATION_S);

	Vector<float> output;
	Vector<float> context;

	timers.set(0);
	timers.set(1);
	while (lifetime < TEST_DURATION_S) {

		task_setup_handler(comms_pipe);

		task_publish_handler(comms_pipe, 0, output, context);
		task_publish_handler(comms_pipe, 1, output, context);

		lifetime += MS_2_S(timers.delay_millis(1, MASTER_CYCLE_TIME_MS));
		errors += assert_leq<float>(timers.millis(1), MASTER_CYCLE_TIME_ERR, "Teensy overcycled"); 
		timers.set(1);
	}

	Serial.println(" * Finished tests");
	Serial.printf(" * %i failed\n", errors + hid_errors);

	return 0;
}


