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

#include "blink.h"
#include <Arduino.h>

uint32_t blinker_timer_mark;
bool blinker_status;

/*
	Easy use blinker implementation
*/
void setup_blink() {
	// Hardware setup
	blinker_status = false;
	blinker_timer_mark = ARM_DWT_CYCCNT;

	pinMode(BLINK_PIN, OUTPUT);
	digitalWrite(BLINK_PIN, blinker_status);
}

void blink_actual(float rate) {
	if ((1E6/float(F_CPU))*(ARM_DWT_CYCCNT - blinker_timer_mark) > rate){
		blinker_timer_mark = ARM_DWT_CYCCNT;
		blinker_status = !blinker_status;
		digitalWrite(BLINK_PIN, blinker_status);
	}
}

/*
	call blink as often as you like, it will toggle at
	min(BLINK_RATE_US, blink_call_rate)

	call blink() much more often than BLINK_RATE_US to get the
	best performance.
*/
void blink(){
	blink_actual(DEFAULT_BLINK_RATE_US);
}


void panic_blink(const char* error) {
	interrupts();
	int cyccnt = ARM_DWT_CYCCNT;
	while(1) {
		blink_actual(PANIC_BLINK_RATE_US);
		if ((1E3/float(F_CPU))*(ARM_DWT_CYCCNT - cyccnt) > 500) {
			printf("%s\n", error);
			cyccnt = ARM_DWT_CYCCNT;
		}
	}
	exit(0);
}