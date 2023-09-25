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
#include "utilities/splash.h"
#include "utilities/vector.h"
#include "utilities/assertions.h"


Vector<float> incremental_vector_fill(int target_length) {
	float tmp[target_length];
	for (int i = 0; i < target_length; i++) {
		tmp[i] = i + 1;
	}

	return Vector<float>(tmp, target_length);
}

int index_operator_test(int target_length) {
	int errors = 0;

	Vector<float> data = incremental_vector_fill(target_length);
	errors += assert_eq<int>(data.size(), target_length, "Vector operator initial size test");
	errors += assert_eq<int>(data.len(), target_length, "Vector operator initial length test");

	for (int i = 0; i < target_length; i++) {
		errors += assert_eq<float>(data[i], float(i+1), "Vector operator test [" + String(i) + "]");
	}

	data.clear();
	errors += assert_eq<int>(data.size(), 0, "Vector operator size test");
	errors += assert_eq<float>(data.as_array(), 0.0f, "Vector fill and Clear test", target_length);

	return errors;
}

int assign_operator_test(Vector<float> v1, Vector<float> v2) {
	int errors = 0;
	
	v1 = v2;
	errors += assert_eq<int>(v1.len(), v2.len(), "Vector Assign and no reset length test");
	errors += assert_eq<int>(v1.size(), v2.size(), "Vector Assign and no reset size test");
	errors += assert_eq<float>(v1.as_array(), v2.as_array(), "Vector Assign test", v1.size());
	
	v2.reset(0);

	errors += assert_neq<int>(v1.len(), 0, "Vector Assign and no reset1 length test");
	errors += assert_neq<int>(v1.size(), 0, "Vector Assign and no reset1 size test");
	errors += assert_eq<int>(v2.len(), 0, "Vector Assign and reset1 length test");
	errors += assert_eq<int>(v2.size(), 0, "Vector Assign and reset1 size test");
	errors += assert_neq<float>(v1.as_array(), 0.0f, "Vector Assign and No reset test", v1.size());
	errors += assert_eq<float>(v2.as_array(), 0.0f, "Vector Assign and reset test", v2.size());
	
	return errors;
}

int reset_and_fill_test(int target_length) {
	Vector<float> data = incremental_vector_fill(target_length);

	data.reset(target_length);
	return assert_eq<float>(data.as_array(), 0.0f, "Vector Reset test", target_length);
	return assert_eq<int>(data.size(), 0, "Vector Reset size test");
}

int append_test(Vector<float> v1) {
	v1.set_items(v1.len());
	int errors = 0;
	int og_length = v1.size();
	float slice1[og_length];
	float slice2[og_length];
	Vector<float> v2 = incremental_vector_fill(v1.size());

	v1.append(&v2);
	errors += assert_eq<int>(v1.size(), 2*og_length, "Vector append size test");
	errors += assert_eq<int>(v1.len(), 2*og_length, "Vector append length test");

	for (int i = 0; i < og_length; i++) {
		assert_eq<float>(v1[i], 0.0f, "Vector append test [" + String(i) + "]");
	}
	for (int i = 0; i < v2.size(); i++) {
		assert_eq<float>(v1[i+og_length], v2[i], "Vector append test [" + String(i) + "]");
	}

	v1.slice(slice1, 0, og_length);
	v1.slice(slice2, og_length, v2.size());
	errors += assert_eq<float>(slice1, 0.0f, "Vector append and slice1 test", og_length);
	errors += assert_eq<float>(v2.as_array(), slice2, "Vector append and slice2 test", v2.size());

	v1.clear();
	errors += assert_eq<int>(v1.size(), 0, "Vector append and clear size test");
	errors += assert_eq<float>(v1.as_array(), 0.0f, "Vector append and clear test", v1.size());
	errors += assert_neq<float>(v2.as_array(), 0.0f, "Vector append and no clear test", v2.size());
	errors += assert_eq<int>(v2.size(), og_length, "Vector append and no clear size test");

	return errors;
}

int iterative_test(int size, int depth) {
	int errors = 0;

	Vector<float> v1(size);
	for (int i = 0; i < depth; i++) {
		Vector<float> v2 = incremental_vector_fill(size + 2);
		v1 = v2;
		errors += assert_eq<float>(v1.as_array(), v2.as_array(), "Vector iterative assign", size);
		errors += assert_eq<int>(v1.size(), v2.size(), "Vector iterative assign v1,v2 size");
		errors += assert_eq<int>(size+2, v2.size(), "Vector iterative assign size,v2 size");
		errors += assert_eq<int>(v1.len(), size + 2, "Vector iterative assign len1");
		errors += assert_eq<int>(v2.len(), size + 2, "Vector iterative assign len2");
	}

	return errors;
}

int recursive_test(Vector<float>* v1, int size, int depth) {
	int errors = 0;

	Vector<float> v2(size);
	errors += assert_eq<float>(v2.as_array(), 0.0f, "Vector recursion init test [" + String(depth) + "]", size);

	for (int i = 0; i < size; i++) {
		v2[i] = float(depth);
	}

	errors += assert_eq<int>(size, v2.size(), "Vector recursive assign size");
	errors += assert_eq<int>(size, v2.len(), "Vector recursive assign len1");

	*v1 = v2;
	errors += assert_eq<int>(v1->size(), v2.size(), "Vector recursive assign size");
	errors += assert_eq<int>(v1->len(), size, "Vector recursive assign len2");
	errors += assert_eq<float>(v1->as_array(), float(depth), "Vector recursion assign test [" + String(depth) + "]", size);

	if (depth <= 0) {
		for (int i = 0; i < size; i++) {
			(*v1)[i] = -1;
		}
		return errors;
	}

	errors += recursive_test(v1, size, depth - 1);

	errors += assert_eq<float>(v1->as_array(), -1, "Vector recursion return check [" + String(depth) + "]", size);

	return errors;
}


int assign_from_pointer(int size) {
	int errors = 0;
	Vector<float>* v1 = new Vector<float>(size);
	Vector<float> v2 = incremental_vector_fill(size);

	v2 = v1;

	for (int i = 0; i < size; i++) {
		(*v1)[i] = float(size);
	}

	for (int i = 0; i < size; i++) {
		v2[i] = float(size) / 2;
	}

	errors += assert_eq<float>(v1->as_array(), float(size), "Pointer assign original modified check", size);
	errors += assert_eq<float>(v2.as_array(), float(size)/2, "Pointer assign assignee check", size);
	delete v1;
	
	v2.clear();
	errors += assert_eq<float>(v2.as_array(), 0.0, "Pointer assign assignee check", size);

	return errors;
}


int pop_test(int size) {
	int errors = 0;
	Vector<float> v = incremental_vector_fill(size);

	for (int i = 0; i < size; i++) {
		float f = v.pop();
		errors += assert_eq<float>(f, i+1, "Vector pop value failed");
		errors += assert_eq<float>(v.size(), size - i - 1, "Vector pop size failed");
	}

	// for (int i = size; i < size + 3; i++) {
	// 	float f = v.pop();
	// 	errors += assert_eq<int>(v.size(), 0, "Vector pop size failed");
	// 	// errors += assert_eq<float>(f, NULL, "Vector pop value failed");
	// 	errors += assert_eq<float>(v.size(), 0, "Vector pop size failed");
	// }

	return errors;
}


Vector<float> test_return() {

	float tmp[9] = {1.0, 2.0, 3.0, -1.0, -2.0, -3.0, 1.0, 1.0, 1.0};
	Vector<float> v(9);
	v.from_array(tmp, 9);
	return v;
}

int main() {
	setup_blink();
	
	unit_test_splash("Vector", -1);

	int n = 5;
	int total_errors = 0;
	// tests index operator, returning vector from function,
	// clearing vector and getting buffer as array
	total_errors += index_operator_test(n);

	// initialize and return incremental vector
	Vector<float> v = incremental_vector_fill(n);

	// Tests assigning and getting buffer as array
	// also clearing and getting buffer as array
	total_errors += assign_operator_test(Vector<float>(n), v);

	// test resetting vector
	total_errors += reset_and_fill_test(n);

	Vector<float> v2(n);
	// test as arg, appending to
	// slicing and clearing vector
	total_errors += append_test(v2);

	// test creating and assigning vector n times
	total_errors += iterative_test(100, 100);

	Vector<float> data(0);
	// test ptr as arg, index assignment,
	// recursively initializing and assigning,
	total_errors += recursive_test(&data, 10, 100);

	total_errors += assign_from_pointer(10);

	total_errors += pop_test(100);

	Serial.printf("=== Finished Vector tests with %i errors ===\n", total_errors);
}