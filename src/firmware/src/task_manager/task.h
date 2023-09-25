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

#ifndef SYS_GRAPH_PROCESS
#define SYS_GRAPH_PROCESS

#include <Arduino.h>
#include "utilities/vector.h"

#define INPUT_DIMENSION 	0
#define PARAM_DIMENSION 	1
#define OUTPUT_DIMENSION 	2
#define TASK_DIMENSIONS 	3

#define TASK_KEY_LENGTH		3


class Task {
	public:
		Vector<int> dimensions;

		Task();
		// int input_dim();
		// int output_dim();
		// int params_dim();

		virtual void reset();
		virtual void clear();
		virtual void print();
		virtual void setup(Vector<float>*);
		virtual void run(Vector<float>*, Vector<float>*, float dt);

		int operator [](int index) { return dimensions[index]; }
};

#endif