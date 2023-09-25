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

#include <Arduino.h>

#ifndef BUFF_BLINKER_H
#define BUFF_BLINKER_H

#define BLINK_PIN LED_BUILTIN
#define DEFAULT_BLINK_RATE_US   250000
#define PANIC_BLINK_RATE_US     50000

/* 

	Use a global rate to blink Teensy's built in LED. 
	Call setup once and then blink() at least as often
	as the rate. Works best when called at 10x the rate.
*/

extern uint32_t blinker_timer_mark;
extern bool blinker_status;

void setup_blink();
void blink();
void panic_blink(const char*);

#endif