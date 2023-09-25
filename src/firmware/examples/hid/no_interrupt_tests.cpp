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

#include "utilities/blink.h"
#include "utilities/timing.h"
#include "utilities/assertions.h"
#include "robot_comms/hid_report.h"

#define MASTER_CYCLE_TIME_US 	1000.0
#define MASTER_CYCLE_TIME_MS 	1.0
#define MASTER_CYCLE_TIME_S 	0.001
#define MASTER_CYCLE_TIME_ERR 	1.001 // ms

FTYK timers;
HidReport report;
IntervalTimer myTimer;

volatile float write_count = 0;
volatile float lifetime = 0;
volatile float pc_write_count = 0;
volatile float pc_lifetime = 0;

void push_hid() {
	report.read();
	pc_write_count = report.get_float(2);
	pc_lifetime = report.get_float(6);
	write_count++;
	report.put_float(2, write_count);
	report.put_float(6, lifetime);
	report.write();
}

// Runs once
void setup() {
	while(!Serial){}

	Serial.println("=== Starting Live HID tests ===");

	while (!usb_rawhid_available()) {} // dont start tests until HID is active
	myTimer.begin(push_hid, MASTER_CYCLE_TIME_US);
}

// Master loop
int main() {
	int total_errors = 0;
	int loops = write_count;
	float prev_write_count = -1;
	setup();
	
	timers.set(0); // Show synchronization between devices
	timers.set(1);
	loops = 0;
	write_count = 0;
	while (lifetime < 5) {
		if (loops % 100 == 0) {
			total_errors += assert_eq<float>(loops, write_count, "loops vs write_count");
			total_errors += assert_eq<float>(pc_write_count - write_count, 0, "pc_write_count - write_count\t");
			total_errors += assert_eq<float>(pc_lifetime - lifetime + timers.secs(1), 0.0, "pc time - mcu time\t\t");
		} 
		assert_eq<float>(prev_write_count, write_count - 1, "Dropped packet\t\t\t");
		prev_write_count = write_count;
		float cycletime = timers.delay_millis(1, MASTER_CYCLE_TIME_MS);
		timers.set(1);
		total_errors += assert_leq<float>(cycletime, MASTER_CYCLE_TIME_ERR, "Teensy overcycled"); 
		lifetime += cycletime / 1000.0;
		loops++;
	}
	
	Serial.printf("=== Finished Live HID tests with %i errors ===\n", total_errors);
	while (1) {
		blink();
	}
}
