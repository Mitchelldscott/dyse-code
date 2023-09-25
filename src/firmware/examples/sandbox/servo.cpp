// #include "unity.h"
#include "utilities/splash.h"
#include "utilities/assertions.h"
#include "motor_drivers/pwm.h"

FTYK timers;

int main() {
	analogWriteFrequency(9, 36621.09);
	analogWriteResolution(12);
	unit_test_splash("Servo Driver", -1);

	Vector<float> input(1);
	Vector<float> output(1);
	Vector<float> parameters(1);

	parameters[0] = 24;

	PwmDriver p;
	p.setup(&parameters);

	int i = 0;
	while (1) {
		timers.set(0);
		input[0] = max(++i % 4096, 0);
		printf("output: %f\n", input[0]);
		// analogWrite(9, input[0]);
		p.run(&input, &output);
		assert_eq<float>(output[0], input[0], "output didn't set");
		timers.delay_millis(0, 10);
	}
}