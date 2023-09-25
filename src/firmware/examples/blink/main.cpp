// #include "unity.h"
#include "utilities/timing.h"
#include "utilities/blink.h"

FTYK timers;

int test_setup_blink(void) {
	// test stuff
	setup_blink();
	return 0;
}

int test_blink(void) {
	// more test stuff
	blink();
	blinker_status = false;
	bool init_status = blinker_status;
	timers.set(0);

	while (init_status == blinker_status) {
		blink();
	}
	// blinker should be precise to 0 microseconds
	if (abs(BLINK_RATE_US - timers.micros(0)) > 1) {
		Serial.println("Failed blink test");
		Serial.printf("%i != %f\n", BLINK_RATE_US, timers.micros(0));
		return 1;
	}
	// TEST_ASSERT_INT32_WITHIN(15, BLINK_RATE_US, timer_info_us(0));
	return 0;
}

int runTests(void) {
	test_setup_blink();
	int errs = test_blink();
	return errs;

	// UNITY_BEGIN();
	// RUN_TEST(test_setup_blink);
	// RUN_TEST(test_blink);
	// return UNITY_END();
}


int main() {
	// Wait ~2 seconds before the Unity test runner
	// establishes connection with a board Serial interface
	while (!Serial) {};

	Serial.println("Start FTYK tests");
	Serial.print(1E9);
	Serial.println();

	timers.set(0);
	timers.set(1);
	timers.set(2);

	timers.delay_millis(0, 1);

	timers.print(0);
	timers.print(1);
	timers.print(2);
	
	Serial.println("Start blink tests");

	int errors = runTests();

	Serial.println("Finished tests");
	Serial.printf("%i failed\n", errors);

	return 0;
}