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

#include "utilities/vector.h"

/// print

template <> void Vector<int>::print() {
	if (items == 0) {
		printf("Vectori [%i]\n", items);
		return;
	}
	printf("Vectori [%i]: [", items);
	for (int i = 0; i < items-1; i++) {
		printf("%i, ", buffer[i]);
	}
	printf("%i]\n", buffer[items-1]);
}

template <> void Vector<float>::print() {
	if (items == 0) {
		printf("Vectorf [%i]\n", items);
		return;
	}
	printf("Vectorf [%i]: [", items);
	for (int i = 0; i < items-1; i++) {
		printf("%f, ", buffer[i]);
	}
	printf("%f]\n", buffer[items-1]);
}
