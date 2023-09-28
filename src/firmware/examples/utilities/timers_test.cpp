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

#define MASTER_CYCLE_TIME_MS 	10
#define MASTER_CYCLE_TIME_S 	(MASTER_CYCLE_TIME_MS * 1E-3)
#define MASTER_CYCLE_TIME_US 	(MASTER_CYCLE_TIME_MS * 1E3)
#define MASTER_CYCLE_TIME_ERR 	(MASTER_CYCLE_TIME_MS + 1)

#define TEST_DURATION_S 		120
#define NUM_TEST_LOOPS			int(TEST_DURATION_S / MASTER_CYCLE_TIME_S)

FTYK timer;
Timestamp timestamp;

// Runs once
int main() {

	int errors = 0;
	float lifetime = 0;

	unit_test_splash("FTYK", TEST_DURATION_S);
	printf(" * Running for %i loops\n", NUM_TEST_LOOPS);

	for(int i = 0; i < 10; i++) {
		int clk1 = ARM_DWT_CYCCNT;
		int clk2 = ARM_DWT_CYCCNT;
		errors += assert_geq<float>(clk2, clk1, "ARM_DWT_CYCCNT test 1v2"); 
		int clk3 = ARM_DWT_CYCCNT;
		errors += assert_gt<float>(clk3, clk2, "ARM_DWT_CYCCNT test 2v3");
		int clk4 = ARM_DWT_CYCCNT;
		errors += assert_gt<float>(clk4, clk3, "ARM_DWT_CYCCNT test 3v4");
		int clk5 = ARM_DWT_CYCCNT;
		errors += assert_gt<float>(clk5, clk4, "ARM_DWT_CYCCNT test 4v5");
	}

	int loop_count = 0;
	float prev_lifetime = 0;

	timer.set();
	timestamp.set();
	int t = micros();
	while (loop_count < NUM_TEST_LOOPS) {

		errors += assert_geq<float>(lifetime, prev_lifetime, "Lifetime value not increasing");

		if (loop_count % int(0.1 * NUM_TEST_LOOPS) == 0) {
			timer.print("Loop");
			timestamp.print();
			printf("Loops: %i\n", loop_count);
			printf("Lifetime: %f\n", lifetime);
		}

		timestamp.update();

		loop_count += 1;
		prev_lifetime = lifetime;
		lifetime += MS_2_S(timer.delay_millis(MASTER_CYCLE_TIME_MS));
		timer.set();
	}

	int t1 = micros();

	printf("\n");
	errors += assert_eq<float>(lifetime, TEST_DURATION_S, 1E-3, "Lifetime != expected");
	errors += assert_eq<float>(lifetime, float(t1 - t) * 1E-6, 1E-6, "Lifetime != micros");
	errors += assert_eq<float>(lifetime, timestamp.total_seconds(), 1E-6, "Lifetime != timestamp");

	timer.print("Loop");
	printf("Loops: %i\n", loop_count);
	printf("Lifetime: %f\n", lifetime);

	printf("\n===== Finished tests, %i failed =====\n", errors);

	return 0;
}