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

#include "timing.h"


FTYK::FTYK() {
	/*
		  Object to get precise timing, nanos is available but not really supported.
		Contains MAX_NUM_TIMERS so different things can be timed simultaneously.

		TODO:
			- add rollover support: ARM_DWT_CYCCNT is a cycle count that resets every 8
			seconds or so, use a member variable to track the number of roll overs.
			This will be dependant on the timer being able to check those rollovers, so
			it will need to be called often (maybe sysgraph needs a timer case check).
	*/
	cyccnt_mark = ARM_DWT_CYCCNT;
}


void FTYK::set() {
	/*
		  Set the timer at idx to the current cycle count.
		@param:
			idx: (int) index of the timer to set.
	*/
	cyccnt_mark = ARM_DWT_CYCCNT;
}

// void FTYK::mark(int idx) {
// 	/*
// 		  Print info about the timer at idx.
// 		@param:
// 			idx: (int) index of the timer to print info about.
// 	*/
// 	print(idx);
// }

float FTYK::cycles() {
	/*
		  Get the number of cycles since last timer.set().
		returns the cycle count as a float to avoid overflows.
		@param:
			idx: (int) index of the timer to get cycles from.
		@return:
			cyccnt: (int) cyles since the timer was set
	*/
	int cyccnt = ARM_DWT_CYCCNT;
	float cycdiff = cyccnt - cyccnt_mark;

	if (cycdiff < 0) {
		cycdiff += MAX_CYCCNT;
	}

	return cycdiff;// accumulator[idx] + (roll_over[idx] * MAX_CYCCNT);
}

float FTYK::nanos() {
	/*
		  Get the number of nanoseconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	return CYCLES_2_NS(cycles());
}

float FTYK::micros() {
	/*
		  Get the number of microseconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	return CYCLES_2_US(cycles());
}

float FTYK::millis() {
	/*
		  Get the number of milliseconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	return CYCLES_2_MS(cycles()); 
}

float FTYK::secs() {
	/*
		  Get the number of seconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	return CYCLES_2_S(cycles());
}


float FTYK::delay_micros(float duration){
	/*
	  Helper to pause for a duration. Duration starts
	when set() is called, which must be called prior.
	@param
		duration: (uint32_t) microseconds to wait (from when set() was called)
	*/
	while(micros() < duration) {}

	return micros();
}

float FTYK::delay_millis(float duration){
	/*
	  Helper to pause for a duration. Duration starts
	when set() is called, which must be called prior.
	@param
		duration: (uint32_t) milliseconds to wait (from when set() was called)
	*/
	
	return US_2_MS(delay_micros(duration * 1E3));
}

void FTYK::print() {
	cycles();
	printf("Timer \t%0.3f|%0.3f|%0.3f|%0.3f (s/ms/us/ns)\n",
		secs(),
		millis(),
		micros(),
		nanos());
}

void FTYK::print(const char* title) {
	printf(title); 
	print();
}


Timestamp::Timestamp() {
	set();
}

void Timestamp::set() {
	timer.set();
	hours = 0;
	minutes = 0;
	seconds = 0;
}

void Timestamp::wrap() {

	if (seconds >= 60) {
		
		minutes += int(seconds / 60);
		seconds = fmod(seconds, 60);

	}

	if (minutes >= 60) {
			
		hours += int(minutes / 60);
		minutes = minutes % 60;

	}

	if (hours >= 24) {

		hours = 0;
	
	}

}

void Timestamp::accumulate(float total_seconds) {
	seconds += total_seconds;
	wrap();
}
		
void Timestamp::update() {

	float timer_seconds = timer.secs();
	timer.set();
	accumulate(timer_seconds);

}

float Timestamp::secs() {
	update();
	return seconds + timer.secs();
}

float Timestamp::total_seconds() {
	return secs() + (60 * (minutes + (60 * hours)));
}

void Timestamp::sync(Timestamp ts) {
	hours = ts.hours;
	minutes = ts.minutes;
	seconds = ts.secs();
	wrap();
}

void Timestamp::sync(float total_seconds) {
	hours = 0;
	minutes = 0;
	seconds = 0;
	accumulate(total_seconds);
}

void Timestamp::print() {
	printf("Timestamp:\t%i:%i:%0.3f (h/m/s)\n", hours, minutes, seconds + timer.secs());
	// timer.print();
}