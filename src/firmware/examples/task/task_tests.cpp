#include "utilities/splash.h"
#include "utilities/timing.h"
#include "utilities/assertions.h"

#include "task_manager/task_factory.h"

FTYK timers;

int simple_proc_test(Task* p) {
	int errors = 0;

	Vector<float> inputs(0);
	Vector<float> outputs(0);
	Vector<float> config(0);

	for (int i = 0; i < 3; i++) {
		timers.set(0);
		p->run(&inputs, &outputs, 1);
		errors += assert_leq<float>(timers.micros(0), 1000, "Run Task timer (us)");
	}

	errors += assert_neq<float>(outputs.as_array(), 0.0f, "task post run non-empty outputs test", outputs.size());

	p->print();
	p->clear();

	return errors;
}

int setup_proc_test(Task* p, Vector<float>* config, int num_inputs) {

	p->print();

	p->setup(config);

	return simple_proc_test(p);

}


int main() {
	int errors = 0;

	unit_test_splash("Task", 0);

	Task p;
	timers.set(0);
	p.reset();
	errors = assert_leq<float>(timers.micros(0), 500, "Reset Task timer (us)");
	errors += simple_proc_test(&p);

	printf("=== Starting LSM9DS1 tests ===\n");

	LSM9DS1 imu;
	
	timers.set(0);
	imu.reset();
	errors += assert_leq<float>(timers.micros(0), 500, "Reset Task timer (us)");
	
	errors += simple_proc_test(&imu);

	printf("=== Starting ComplimentaryFilter tests ===\n");

	ComplimentaryFilter cmf;

	Vector<float> cmf_config(0);
	cmf_config.push(0.6);
	
	timers.set(0);
	cmf.reset();
	errors += assert_leq<float>(timers.micros(0), 500, "Reset Task timer (us)");

	errors += setup_proc_test(&cmf, &cmf_config, 12);

	printf("=== Starting Factory tests ===\n");

	Vector<Task*> p_list(2);
	p_list.push(new_task("DS1"));
	p_list.push(new_task("CMF"));
	p_list.print();

	printf("=== LSM9DS1 ===\n");
	errors += simple_proc_test(p_list[0]);
	printf("=== ComplimentaryFilter ===\n");
	errors += setup_proc_test(p_list[1], &cmf_config, 12);

	printf("=== Finished task tests === %i errors\n", errors);

	return 0;
}