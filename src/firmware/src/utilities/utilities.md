# Utilities

Miscellaneous code to help with firmware implementations

## Assertions
### Description
Assertions are macros and functions to provide numerical comparisons that print custom messages.

### Usuage
	#include "utilities/assertions.h"

	int errors = assert_eq<int>(0, 0, 	"int equal true test");
	errors += assert_eq<int>(   0, 1, 	"int equal false test");
	errors += assert_neq<int>(  0, 0,   "int not equal false test");

### Tests cases
 - True positive  (int, byte, float and char for all functions)
 - False positive (int, byte, float and char for all functions)
 - True negative  (int, byte, float and char for all functions)
 - False negative (int, byte, float and char for all functions)
 - Timing evaluation of each call

### TODO
 - Use utilities/blink to have an err_blink() (resolve blink updates first)

## Blink
### Description
Basic functions to drive the onboard LED.

### Usuage
	#include "utilities/blink.h"

	setup_blink();
	blink();

### Tests cases
 - Timing evaluation of functions
 - Visually confirm LED blinking ~= set rate

### TODO
 - Add setup argument to control the blink rate

## Splash
### Description
Prints the Dyse Industries splash for a variety of scenarios.

### Usuage
	#include "utilities/splash.h"
	#define TEST_DURATION_S 30
	splash();
	unit_test_splash("Unit Test name", TEST_DURATION_S); // waits for a serial monitor

### Tests cases
 - Visually confirm print looks aesthetic

## Timing
### Description
A timer object that can return cycles, nanos, millis and secs. Also has easy functions to wait for a timer to reach a duration.

### Usuage
	#include "utilities/timing.h"

	FTYK timers;
	timer.set(0);
	while (timer.millis(0) < 1000) {
		timer.set(1);
		timer.delay_millis(100);
		printf("Printing every 100ms for 1000ms");
	}
	printf("Done");

### Tests cases
 - Comparisons with millis() call
 - Long duration timing (>30s)
 - accumulating timer.secs(1) vs letting timer[0] accumulate
 - longest duration between calls to timer members to catch overflows (~6s)

## Vector
### Description
Array of objects with dynamic resizing. Cause it's better than standard C arrays.

### Usuage
	#include "utilities/vector.h"

	Vector<float> v1(10);
	if (v1.size() == 10) {
		v1.print();
	}
	v1[3] = 1;
	if (v1.find(1) == 3) {
		Vector<foat> v2 = v1;
		v2.print();
	}

### Tests cases
 - Timing evaluation of functions
 - Create and modify vector
 - Create two vectors (v1, v2) (use multiple methods: insert, append, push and from_array)
   - modify v1 (non-zero)
   - v2 = v1
   - v1.clear() // assert v1 is now zero
   - assert v2 is not
 - Repeat above for v2 and v1 as pointer (passed as args to function, (*v1, v2), (v1, *v2), (*v1, *v2))
