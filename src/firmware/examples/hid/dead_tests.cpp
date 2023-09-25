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

#define MASTER_CYCLE_TIME_MS 	10
#define MASTER_CYCLE_TIME_S 	(MASTER_CYCLE_TIME_MS * 1E-3)
#define MASTER_CYCLE_TIME_US 	(MASTER_CYCLE_TIME_MS * 1E3)
#define MASTER_CYCLE_TIME_ERR 	(MASTER_CYCLE_TIME_MS + 1)

#define TEST_DURATION_S			30

FTYK timers;


// Runs once
int main() {

	int errors = 0;
	float lifetime = 0;

	unit_test_splash("Dead comms", TEST_DURATION_S);

	CommsPipeline* comms_pipe = enable_hid_interrupts();
	while (!usb_rawhid_available()) {};

	timers.set(0);
	timers.set(1);
	while (lifetime < TEST_DURATION_S) {

		float cycletime = timers.delay_millis(1, MASTER_CYCLE_TIME_MS);
		timers.set(1);
		lifetime += MS_2_S(cycletime);
		errors += assert_leq<float>(cycletime, MASTER_CYCLE_TIME_ERR, "Teensy overcycled (ms)"); 		
	}

	printf(" * Finished tests\n");
	printf(" * %i failed\n", int(errors + hid_errors));

	while(1){};
	return 0;
}


