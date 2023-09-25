#ifndef COMP_FILTER_H
#define COMP_FILTER_H

#include "utilities/timing.h"
#include "task_manager/task.h"
#include "tasks/linear_algebra.h"

#define ATTITUDE_DIM 	3
#define CMF_STATE_SIZE 	6
#define CMF_INPUT_DIMS 	12
#define DEFAULT_GAIN 	0.4

class ComplimentaryFilter: public Task {
	private:
		char key[3] = {'C', 'M', 'F'};
		FTYK timers;
		float axz_norm = 0;
		float ayz_norm = 0;
		float mag_norm = 0;
		float K = DEFAULT_GAIN;
		float q_gyro[ATTITUDE_DIM];
		float q_accel[ATTITUDE_DIM];
		float bias_accel[ATTITUDE_DIM];

	public:
		ComplimentaryFilter() {
			dimensions.reset(TASK_DIMENSIONS);
			dimensions[INPUT_DIMENSION] = CMF_INPUT_DIMS;
			dimensions[PARAM_DIMENSION] = 1;
			dimensions[OUTPUT_DIMENSION] = ATTITUDE_DIM;

			timers.set(0);
			reset();
		}

		void setup(Vector<float>* config) {
			K = (*config)[0];
			timers.set(0);
			for (int i = 0; i < ATTITUDE_DIM; i++) {
				q_accel[i] = 0;
				q_gyro[i] = 0;
			}
		}

		void filter(float* accel, float* gyro, float* mag, float dt, float* estimate) {
			float axz[2] = {accel[0], accel[2]};
			float ayz[2] = {accel[1], accel[2]};
			axz_norm = nd_norm(axz, ATTITUDE_DIM - 1);
			ayz_norm = nd_norm(ayz, ATTITUDE_DIM - 1);
			mag_norm = nd_norm(mag, ATTITUDE_DIM);

			// Calulate attitude using accelerations + magnetometer and trig
			q_accel[0] = atan2(accel[1], axz_norm);
			q_accel[1] = atan2(-accel[0], ayz_norm);

			float opposite = ((mag[2]*sin(q_accel[0])) - (mag[1]*cos(q_accel[0]))) / mag_norm;
			float adjacent = ((mag[0]*cos(q_accel[1])) + (sin(q_accel[1]) * (mag[1]*cos(q_accel[0])) + (mag[2]*sin(q_accel[0])))) / mag_norm;

			q_accel[2] = atan2(opposite, adjacent);
			if (isnan(q_accel[2])) {
				q_accel[2] = 0;
			}

			// integrate gyro measurements over time
			weighted_vector_addition(estimate, gyro, 1, dt, 3, q_gyro);

			// fuse the accel and gyro estimates
			weighted_vector_addition(q_accel, q_gyro, K, 1-K, 3, estimate);
		}

		void reset() {
			K = DEFAULT_GAIN;
			clear();
		}

		void clear() {
			axz_norm = 0;
			ayz_norm = 0;
			mag_norm = 0;
			for (int i = 0; i < ATTITUDE_DIM; i++) {
				q_gyro[i] = 0;
				q_accel[i] = 0;
				bias_accel[i] = 0;
			}
		}

		void run(Vector<float>* inputs, Vector<float>* outputs, float dt) {
			
			float estimate[ATTITUDE_DIM];
			float accel[ATTITUDE_DIM];
			float gyro[ATTITUDE_DIM];
			float mag[ATTITUDE_DIM];
			inputs->slice(estimate, 0, ATTITUDE_DIM);
			inputs->slice(accel, 3, ATTITUDE_DIM);
			inputs->slice(gyro, 6, ATTITUDE_DIM);
			inputs->slice(mag, 9, ATTITUDE_DIM);
			
			filter(accel, gyro, mag, dt, estimate);

			outputs->from_array(estimate, dimensions[OUTPUT_DIMENSION]);
		}

		void print() {
			Serial.println("ComplimentaryFilter");
			Serial.printf("\tgain: [%f]\n", K);
			Serial.printf("\tq_accel: [%f, %f, %f]\n", q_accel[0], q_accel[1], q_accel[2]);
			Serial.printf("\tq_gyro: [%f, %f, %f]\n", q_gyro[0], q_gyro[1], q_gyro[2]);
		}
};

#endif