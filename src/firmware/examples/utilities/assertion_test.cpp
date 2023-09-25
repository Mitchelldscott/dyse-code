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
#include "utilities/blink.h"
#include "utilities/assertions.h"

#define EXPECTED_ERRORS 8

int main() {
	setup_blink();
	while(!Serial){}

	Serial.println("=== Starting Assertion tests ===");

	int errors = assert_eq<int>(0, 0, 	"int equal true test");
	errors += assert_eq<int>(   0, 1, 	"int equal false test");
	errors += assert_neq<int>(  0, 0,   "int not equal false test");
	errors += assert_neq<int>(  0, 1,   "int not equal true test");
	errors += assert_leq<int>(  1, 0, 	"int less than or equal false test");
	errors += assert_leq<int>(  0, 1, 	"int less than or equal true test");
	errors += assert_leq<int>(  0, 0, 	"int less than or equal true test");
	errors += assert_geq<int>(  0, 1, 	"int greater than or equal false test");
	errors += assert_geq<int>(  1, 0, 	"int greater than or equal true test");
	errors += assert_geq<int>(  0, 0, 	"int greater than or equal true test");

	errors += assert_eq<float>( 0.0, 0.0, "float equal true test");
	errors += assert_eq<float>( 0.0, 1.0, "float equal false test");
	errors += assert_neq<float>(0.0, 0.0, "float not equal false test");
	errors += assert_neq<float>(0.0, 1.0, "float not equal true test");
	errors += assert_leq<float>(1.0, 0.0, "float less than or equal false test");
	errors += assert_leq<float>(0.0, 1.0, "float less than or equal true test");
	errors += assert_leq<float>(0.0, 0.0, "float less than or equal true test");
	errors += assert_geq<float>(1.0, 0.0, "float greater than or equal false test");
	errors += assert_geq<float>(0.0, 1.0, "float greater than or equal true test");
	errors += assert_geq<float>(0.0, 0.0, "float greater than or equal true test");

	Serial.printf("=== Finished Assertion tests with %i errors ===\n", errors - EXPECTED_ERRORS);
	while (1) {
		blink();
	}
}