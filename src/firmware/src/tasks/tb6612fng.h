#ifndef TB6612FNG_H
#define TB6612FNG_H

#include "utilities/timing.h"
#include "task_manager/task.h"

#define FNG_WRITE_RESOLUTION 15
#define FNG_RESOLUTION_SCALE 32768

class Tb6612Fng: public Task {
	private:
		char key[3] = {'F', 'N', 'G'};
		int stby;
		int ai1;
		int ai2;
		int bi1;
		int bi2;
		int pwma;
		int pwmb;


	public:
		Tb6612Fng() {
			dimensions[INPUT_DIMENSION] = 1;
			dimensions[PARAM_DIMENSION] = 7;
			dimensions[OUTPUT_DIMENSION] = 4;

			reset();
		}

		void setup(Vector<float>* config) {

			stby = (*config)[0];
			ai1 = (*config)[1];
			ai2 = (*config)[2];
			bi1 = (*config)[3];
			bi2 = (*config)[4];
			pwma = (*config)[5];
			pwmb = (*config)[6];

			pinMode(stby, OUTPUT);
			pinMode(ai1, OUTPUT);
			pinMode(ai2, OUTPUT);
			pinMode(bi1, OUTPUT);
			pinMode(bi2, OUTPUT);
			pinMode(pwma, OUTPUT);
			pinMode(pwmb, OUTPUT);

			analogWriteFrequency(pwma, 1000.0);
			analogWriteFrequency(pwmb, 1000.0);
			analogWriteResolution(FNG_WRITE_RESOLUTION);
		}

		void reset() {

			digitalWrite(stby, 0);
			digitalWrite(ai1, 0);
			digitalWrite(bi1, 0);
			digitalWrite(ai2, 0);
			digitalWrite(bi2, 0);

			analogWrite(pwma, 0);
			analogWrite(pwmb, 0);
		}

		void clear() {

		}

		void run(Vector<float>* inputs, Vector<float>* outputs, float dt) {
			// printf("analogWrite[%i] %f\n", pin, (*inputs)[0]);
			int direction = (*inputs)[0] > 0;
			int drive_speed = int(max(0.0, min(1.0, abs((*inputs)[0]))) * FNG_RESOLUTION_SCALE);
			
			digitalWrite(stby, 1);
			digitalWrite(ai1, direction);
			digitalWrite(bi1, direction);
			digitalWrite(ai2, !direction);
			digitalWrite(bi2, !direction);

			analogWrite(pwma, drive_speed);
			analogWrite(pwmb, drive_speed);

			(*outputs)[0] = 1;
			(*outputs)[1] = direction;
			(*outputs)[2] = drive_speed;
			(*outputs)[3] = drive_speed;
		}

		void print() {
			// Serial.println("PWM Driver");
			// Serial.printf("\tPin: %i\n", pin);
			// Serial.printf("\tOutput: %i\n", output);
		}
};

#endif
