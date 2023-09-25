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

#ifndef TASK_FACTORY
#define TASK_FACTORY

#include "tasks/pwm.h"
#include "tasks/lsm9ds1.h"
#include "tasks/lsm6dsox.h"
#include "tasks/sin_task.h"
#include "tasks/constant_task.h"
#include "tasks/complimentary_filter.h"

#define LSM6DSOX_DRIVER_KEY "LSM"
#define LSM9DS1_DRIVER_KEY "DS1"
#define PWM_DRIVER_KEY      "PWM"
#define COMPFLTR_DRIVER_KEY "CMF"
#define CONSTANT_DRIVER_KEY "VAL"
#define SINUSIOD_DRIVER_KEY "SIN"

Task* new_task(const char*);

#endif