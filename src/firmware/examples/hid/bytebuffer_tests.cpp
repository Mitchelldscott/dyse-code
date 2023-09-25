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
#include "utilities/timing.h"
#include "utilities/assertions.h"
#include "hid_comms/bytebuffer.h"

#define DEMO_BUFFER_SIZE 64

FTYK timer;

int dummy_tests() {
	int errors = 0;

	int* output = new int[5];
	int value[5] = {8, 8, 8, 8};
	memcpy(output, value, 5 * sizeof(int));

	errors += assert_eq<int>(output, value, "memcpy int check", 5);

	delete [] output;

	int test = 0;
	unsigned char* p = (unsigned char*) &test;
	for (unsigned int i = 0; i < sizeof(int); i++) {
		p[i] = 8;
	}

	float f_test_val = 0;
	char float_val[4] = {0x78, 0xEC, 0x7F, 0x3F};//{0x40, 0x9F, 0xC1, 0x31};
	memcpy(&f_test_val, float_val, 4);
	assert_eq<float>(f_test_val, 4.99, "float memcpy test");

	float f_test_val2 = 0.9997;
	char float_val2[4] = {0x78, 0xEC, 0x7F, 0x3F};//{0x40, 0x9F, 0xC1, 0x31};
	memcpy(float_val2, &f_test_val2, 4);
	assert_eq<char>(float_val2, float_val, "float memcpy test", 4);

	return errors;
}

int buffer_put_get_scalar() {
	int errors = 0;
	ByteBuffer<DEMO_BUFFER_SIZE> buffer;
	buffer.clear();

	int constant_int = 0x13;//0x88888888;
	byte constant_byte = 0x88;
	float constant_float = 13.13;

	int iresults[int(DEMO_BUFFER_SIZE / sizeof(int))];
	float fresults[int(DEMO_BUFFER_SIZE / sizeof(float))];

	// int put
	timer.set(0);
	for (int i = 0; i < DEMO_BUFFER_SIZE; i+=sizeof(int)) {
		buffer.put<int>(i, constant_int);
	}
	timer.print(0, "int put");

	// int get
	timer.set(0);
	for (int i = 0; i < int(DEMO_BUFFER_SIZE / sizeof(int)); i++) {
		iresults[i] = buffer.get<int>(i * sizeof(int));
	}
	timer.print(0, "int get");

	// check
	errors += assert_eq<int>(iresults, constant_int, "failed packet put/get test for constant int", int(DEMO_BUFFER_SIZE / sizeof(int)));

	buffer.clear();
	
	timer.set(0);
	for (int i = 0; i < DEMO_BUFFER_SIZE; i++) {
		buffer.put(i, constant_byte);
	}
	timer.print(0, "byte put");

	timer.set(0);
	byte* tmp = buffer.buffer();
	timer.print(0, "byte buffer get all");

	// check
	errors += assert_eq<byte>(tmp, constant_byte, "failed packet put test for constant byte", DEMO_BUFFER_SIZE);

	// float put
	timer.set(0);
	for (int i = 0; i < DEMO_BUFFER_SIZE; i+=sizeof(float)) {
		buffer.put<float>(i, constant_float);
	}
	timer.print(0, "float put");

	// float get
	timer.set(0);
	for (int i = 0; i < int(DEMO_BUFFER_SIZE / sizeof(float)); i++) {
		fresults[i] = buffer.get<float>(i * sizeof(float));
	}
	timer.print(0, "float get");

	// check
	errors += assert_eq<float>(fresults, constant_float, "failed packet put/get test for constant float", int(DEMO_BUFFER_SIZE / sizeof(float)));
	
	return errors;
}

int buffer_put_get_array() {
	int errors = 0;
	ByteBuffer<DEMO_BUFFER_SIZE> buffer;
	buffer.clear();

	int n_ints = int(DEMO_BUFFER_SIZE / sizeof(int));
	int n_floats = int(DEMO_BUFFER_SIZE / sizeof(float));

	int constant_int = 0x13;//0x88888888;
	byte constant_byte = 0x88;
	float constant_float = 13.13;

	int array_int[n_ints];
	byte array_byte[DEMO_BUFFER_SIZE];
	float array_float[n_floats];

	for (int i = 0; i < n_ints; i++) {
		array_int[i] = constant_int;
	}
	for(int i = 0; i < DEMO_BUFFER_SIZE; i++) {
		array_byte[i] = constant_byte;
	}
	for (int i = 0; i < int(DEMO_BUFFER_SIZE / sizeof(float)); i++) {
		array_float[i] = constant_float;
	}

	int iresults[n_ints];
	float fresults[n_floats];

	// int put
	timer.set(0);
	buffer.put<int>(0, n_ints, array_int);
	timer.print(0, "int put");
	// int get
	timer.set(0);
	buffer.get<int>(0, n_ints, iresults);
	timer.print(0, "int get");
	// check
	errors += assert_eq<int>(iresults, constant_int, "failed packet put/get test for array int", int(DEMO_BUFFER_SIZE / sizeof(int)));


	buffer.clear();
	
	// byte put
	timer.set(0);
	buffer.put(0, DEMO_BUFFER_SIZE, array_byte);
	timer.print(0, "byte put");
	// byte get
	timer.set(0);
	byte* tmp = buffer.buffer();
	timer.print(0, "byte buffer get all");
	// check
	errors += assert_eq<byte>(tmp, constant_byte, "failed packet put test for array byte", DEMO_BUFFER_SIZE);

	buffer.clear();

	// float put
	timer.set(0);
	buffer.put<float>(0, n_floats, array_float);
	timer.print(0, "float put");
	// float get
	timer.set(0);
	buffer.get<float>(0, n_floats, fresults);
	timer.print(0, "float get");
	// check
	errors += assert_eq<float>(fresults, constant_float, "failed packet put/get test for array float", n_floats);
	
	return errors;
}

// Runs once
int main() {
	wait_for_serial();
	Serial.println("========= ByteBuffer Tests ==========");
	
	int errors = dummy_tests();
	// errors += buffer_put_get_scalar();
	// errors += buffer_put_get_array();

	Serial.println("Finished tests");
	Serial.printf("%i failed\n", errors);

	return 0;
}