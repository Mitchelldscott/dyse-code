#ifndef LSM9DS1_DRIVER_H
#define LSM9DS1_DRIVER_H

#include "Adafruit_LSM9DS1.h"
#include "task_manager/task.h"

#define LSM9DS1_MOSI    26
#define LSM9DS1_SCK     27
#define LSM9DS1_XGCS    37
#define LSM9DS1_MCS     38
#define LSM9DS1_MISO    39


class LSM9DS1: public Task {
	private:
		char key[3] = {'D', 'S', '1'};
		sensors_event_t accel;
		sensors_event_t mag;
		sensors_event_t gyro;
		sensors_event_t temp;
    	Adafruit_LSM9DS1 lsm;

	public:
		LSM9DS1() {
			dimensions.reset(TASK_DIMENSIONS);
			dimensions[INPUT_DIMENSION] = 0;
			dimensions[PARAM_DIMENSION] = 0;
			dimensions[OUTPUT_DIMENSION] = 9;

			lsm = Adafruit_LSM9DS1(LSM9DS1_SCK, LSM9DS1_MISO, LSM9DS1_MOSI, LSM9DS1_XGCS, LSM9DS1_MCS);

			if (!lsm.begin())
			{
				printf("Oops ... unable to initialize the LSM9DS1. Check your wiring!\n");
				while (1);
			}
		}

		void setup(Vector<float>* config) {
			// 1.) Set the accelerometer range
			// lsm.setupAccel(lsm.LSM9DS1_ACCELRANGE_2G);
			//lsm.setupAccel(lsm.LSM9DS1_ACCELRANGE_4G);
			//lsm.setupAccel(lsm.LSM9DS1_ACCELRANGE_8G);
			lsm.setupAccel(lsm.LSM9DS1_ACCELRANGE_16G);

			// 2.) Set the magnetometer sensitivity
			lsm.setupMag(lsm.LSM9DS1_MAGGAIN_8GAUSS);
			//lsm.setupMag(lsm.LSM9DS1_MAGGAIN_8GAUSS);
			//lsm.setupMag(lsm.LSM9DS1_MAGGAIN_12GAUSS);
			//lsm.setupMag(lsm.LSM9DS1_MAGGAIN_16GAUSS);

			// 3.) Setup the gyroscope
			// lsm.setupGyro(lsm.LSM9DS1_GYROSCALE_200DPS);
			//lsm.setupGyro(lsm.LSM9DS1_GYROSCALE_500DPS);
			lsm.setupGyro(lsm.LSM9DS1_GYROSCALE_2000DPS);
		}

		void reset() {
			// value = 0;
		}

		void clear() {
			// value = 0;
		}

		void run(Vector<float>* inputs, Vector<float>* outputs, float dt) {
			lsm.read();
    		lsm.getEvent(&accel, &mag, &gyro, &temp);
    		(*outputs)[0] = accel.acceleration.x;
    		(*outputs)[1] = accel.acceleration.y;
    		(*outputs)[2] = accel.acceleration.z;
    		(*outputs)[3] = mag.magnetic.x;
    		(*outputs)[4] = mag.magnetic.y;
    		(*outputs)[5] = mag.magnetic.z;
    		(*outputs)[6] = gyro.gyro.x;
    		(*outputs)[7] = gyro.gyro.y;
    		(*outputs)[8] = gyro.gyro.z;
		}

		void print() {
			printf("LSM9DS1\n");
			printf("Accel X: %f m/s^2\n", accel.acceleration.x);
		    printf("\tY: %f m/s^2\n", accel.acceleration.y);
		    printf("\tZ: %f m/s^2\n", accel.acceleration.z);

		    printf("Mag X: %f uT\n", mag.magnetic.x);
		    printf("\tY: %f uT\n", mag.magnetic.y);  
		    printf("\tZ: %f uT\n", mag.magnetic.z);  

		    printf("Gyro X: %f rad/s\n", gyro.gyro.x);
		    printf("\tY: %f rad/s\n", gyro.gyro.y);   
		    printf("\tZ: %f rad/s\n", gyro.gyro.z); 
		}
};

// class Lsm9ds1: public Task {
// 	private:
// 		char key[3] = {'N', 'C', 'V'};
// 		int n;
// 		float* buffer; 


// 	public:
// 		Lsm9ds1() {
// 			dimensions.reset(TASK_DIMENSIONS);
// 			dimensions[INPUT_DIMENSION] = 0;
// 			dimensions[PARAM_DIMENSION] = 0;
// 			dimensions[OUTPUT_DIMENSION] = 0;

// 			reset();
// 		}

// 		void setup(Vector<float>* config) {
// 			n = config->size();
// 			buffer = config->as_array();
// 			dimensions[PARAM_DIMENSION] = n;
// 			dimensions[OUTPUT_DIMENSION] = n;
// 		}

// 		void reset() {
// 			value = 0;
// 		}

// 		void clear() {
// 			value = 0;
// 		}

// 		void run(Vector<float>* inputs, Vector<float>* outputs) {
			
// 			(*outputs)[0] = float(value);
// 		}

// 		void print() {
// 			Serial.println("Constant Value task");
// 			Serial.printf("\tOutput: %i\n", value);
// 		}
// };

#endif