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

#include "utilities/splash.h"
#include "utilities/assertions.h"
#include "task_manager/task_manager.h"

#define MASTER_CYCLE_TIME_MS 	0.5
#define MASTER_CYCLE_TIME_S 	(MASTER_CYCLE_TIME_MS * 1E-3)
#define MASTER_CYCLE_TIME_US 	(MASTER_CYCLE_TIME_MS * 1E3)
#define MASTER_CYCLE_TIME_ERR 	(MASTER_CYCLE_TIME_MS + 0.01)

FTYK timers;

// Runs once
void setup() {
	splash();
	timers.set(0);
	init_task_manager();
}

// Master loop
int main() {
	setup();

	timers.set(1);
	while (1) {
	
		spin();
	
		assert_leq<float>(timers.delay_millis(1, MASTER_CYCLE_TIME_MS), MASTER_CYCLE_TIME_ERR, "Teensy overcycled"); // timer error threashold (very tight)
		timers.set(1);
	}

	return 0;
}
