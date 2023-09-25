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
	for (size_t i = 0; i < MAX_NUM_TIMERS; i++) {
		cyccnt_mark[i] = ARM_DWT_CYCCNT;
		seconds[i] = 0.0;
		minutes[i] = 0;
		hours[i] = 0;
	}
}


void FTYK::set(int idx) {
	/*
		  Set the timer at idx to the current cycle count.
		@param:
			idx: (int) index of the timer to set.
	*/
	cyccnt_mark[idx] = ARM_DWT_CYCCNT;
	active_timers[idx] = 1;
	seconds[idx] = 0.0;
	minutes[idx] = 0;
	hours[idx] = 0;
}

void FTYK::mark(int idx) {
	/*
		  Print info about the timer at idx.
		@param:
			idx: (int) index of the timer to print info about.
	*/
	print(idx);
}

void FTYK::accumulate_cycles(int idx, int cycles) {
	seconds[idx] += CYCLES_2_S(cycles);

	while (seconds[idx] >= 60) {
		seconds[idx] -= 60.0;
		minutes[idx] += 1;
		if (minutes[idx] >= 60) {
			minutes[idx] -= 60;
			hours[idx] += 1;
		}
	}
}

void FTYK::cycles(int idx) {
	/*
		  Get the number of cycles since last timer.set().
		returns the cycle count as a float to avoid overflows.
		@param:
			idx: (int) index of the timer to get cycles from.
		@return:
			cyccnt: (int) cyles since the timer was set
	*/
	int cyccnt = ARM_DWT_CYCCNT;
	int cycdiff = cyccnt - cyccnt_mark[idx];
	cyccnt_mark[idx] = cyccnt;
	
	if (cycdiff < 0) {
		cycdiff += MAX_CYCCNT;
	}

	accumulate_cycles(idx, cycdiff);
}

float FTYK::nanos(int idx) {
	/*
		  Get the number of nanoseconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	cycles(idx);
	return S_2_NS(seconds[idx]);
}

float FTYK::micros(int idx) {
	/*
		  Get the number of microseconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	cycles(idx);
	return S_2_US(seconds[idx]);
}

float FTYK::millis(int idx) {
	/*
		  Get the number of milliseconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	cycles(idx);
	return S_2_MS(seconds[idx]);
}

float FTYK::secs(int idx) {
	/*
		  Get the number of seconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	cycles(idx);
	return seconds[idx];
}

int FTYK::mins(int idx) {
	return minutes[idx];
}

int FTYK::hrs(int idx) {
	return hours[idx];
}

float FTYK::total_seconds(int idx) {
	/*
		  Get the number of seconds since last timer.set().
		@param:
			idx: (int) index of the timer to get cycles from.
	*/
	cycles(idx);
	return seconds[idx] + (60 * (minutes[idx] + (60 * hours[idx])));
}


float FTYK::delay_micros(int idx, float duration){
	/*
	  Helper to pause for a duration. Duration starts
	when set() is called, which must be called prior.
	@param
		duration: (uint32_t) microseconds to wait (from when set() was called)
	*/
	static int i = 0;
	while(micros(idx) < duration) {
		cycles(i);
		i = (i + 1) % MAX_NUM_TIMERS;
	}

	return S_2_US(seconds[idx]);
}

float FTYK::delay_millis(int idx, float duration){
	/*
	  Helper to pause for a duration. Duration starts
	when set() is called, which must be called prior.
	@param
		duration: (uint32_t) milliseconds to wait (from when set() was called)
	*/
	static int i = 0;
	while(millis(idx) < duration) {
		cycles(i);
		i = (i + 1) % MAX_NUM_TIMERS;
	}
	
	return S_2_MS(seconds[idx]);
}

void FTYK::print(int idx) {
	cycles(idx);
	printf("Timer %i\n", idx);
	printf("%i:%i:%f\n", 
		hours[idx],
		minutes[idx],
		seconds[idx]);
}

void FTYK::print(int idx, const char* title) {
	cycles(idx);
	printf(title); printf(" Timer %i\n", idx);
	printf("%i:%i:%f\n", 
		hours[idx],
		minutes[idx],
		seconds[idx]);
}