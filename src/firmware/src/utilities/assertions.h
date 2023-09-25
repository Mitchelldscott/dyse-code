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

#ifndef BUFF_LOGGERS
#define BUFF_LOGGERS
/*
	Plans for this is to become like unity macros
	- used for assertions and debug printing
	- need to support unit testing
	- if it gets big enough start making a fancy serial layer (PC side)
*/

#define TEST_INFO_SCALAR(msg, a) { \
	Serial.print("[TEST ERROR]\t"); \
	Serial.print(msg); \
	Serial.printf(": %.6f\n", float(a)); \
}

#define TEST_INFO(msg, op, a, b) { \
	Serial.print("[TEST ERROR]\t"); \
	Serial.print(msg); \
	Serial.printf(": %.6f %s %.6f\n", float(a), op, float(b)); \
}

template <typename T> int assert_eq(T a, T b, String message) {
	if (abs(a - b) > 1E-15) {
		TEST_INFO(message, "!=", a, b);
		return 1;
	}
	return 0;
}

template <typename T> int assert_eq(T* a, T b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_eq(a[i], b, message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_eq(T* a, T* b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_eq(a[i], b[i], message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_eq(T a, T b, T tol, String message) {
	if (a > b + tol) {
		TEST_INFO(message, "!<=", a, b+tol);
		return 1;
	}
	if (a < b - tol) {
		TEST_INFO(message, "!>=", a, b-tol);
		return 1;
	}
	return 0;
}

template <typename T> int assert_neq(T a, T b, String message) {
	if (a == b) {
		TEST_INFO(message, "==", a, b);
		return 1;
	}
	return 0;
}

template <typename T> int assert_neq(T* a, T b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_neq(a[i], b, message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_neq(T* a, T* b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_eq(a[i], b[i], message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_leq(T a, T b, String message) {
	if (a > b) {
		TEST_INFO(message, "!<=", a, b);
		return 1;
	}
	return 0;
}

template <typename T> int assert_leq(T* a, T b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_leq(a[i], b, message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_leq(T* a, T* b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_leq(a[i], b[i], message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_lt(T a, T b, String message) {
	if (a >= b) {
		TEST_INFO(message, "!<=", a, b);
		return 1;
	}
	return 0;
}

template <typename T> int assert_lt(T* a, T b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_lt(a[i], b, message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_lt(T* a, T* b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_lt(a[i], b[i], message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_geq(T a, T b, String message) {
	if (a < b) {
		TEST_INFO(message, "!>=", a, b);
		return 1;
	}
	return 0;
}

template <typename T> int assert_geq(T* a, T b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_geq(a[i], b, message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_geq(T* a, T* b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_geq(a[i], b[i], message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_gt(T a, T b, String message) {
	if (a <= b) {
		TEST_INFO(message, "!>", a, b);
		return 1;
	}
	return 0;
}

template <typename T> int assert_gt(T* a, T b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_gt(a[i], b, message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_gt(T* a, T* b, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_gt(a[i], b[i], message + " [" + String(i) + "]");
	}
	return errors;
}

template <typename T> int assert_not_nan(T a){
	if (isnan(a)) {
		TEST_INFO_SCALAR("Value is NaN", a);
		return 1;
	}
	return 0;
}

template <typename T> int assert_not_nan(T* a, String message, int n) {
	int errors = 0;
	for (int i = 0; i < n; i++) {
		errors += assert_not_nan(a[i], message + " [" + String(i) + "]");
	}
	return errors;
}

#endif