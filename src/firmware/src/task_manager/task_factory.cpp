#include "task_manager/task_factory.h"



Task* new_task(const char* key){
    if (key[0] == LSM6DSOX_DRIVER_KEY[0] && key[1] == LSM6DSOX_DRIVER_KEY[1] && key[2] == LSM6DSOX_DRIVER_KEY[2]) {
        // printf("Task builder LSM6DSOX\n");
        return new LSM6DSOX();
    }
    if (key[0] == LSM9DS1_DRIVER_KEY[0] && key[1] == LSM9DS1_DRIVER_KEY[1] && key[2] == LSM9DS1_DRIVER_KEY[2]) {
        // printf("Task builder LSM6DSOX\n");
        return new LSM9DS1();
    }
    else if (key[0] == PWM_DRIVER_KEY[0] && key[1] == PWM_DRIVER_KEY[1] && key[2] == PWM_DRIVER_KEY[2]) {
        // printf("Task builder PWM_DRIVER_KEY\n");
        return new PwmDriver();
    }
    else if (key[0] == COMPFLTR_DRIVER_KEY[0] && key[1] == COMPFLTR_DRIVER_KEY[1] && key[2] == COMPFLTR_DRIVER_KEY[2]) {
        // printf("Task builder COMPFLTR_DRIVER_KEY\n");
        return new ComplimentaryFilter();
    }
    else if (key[0] == CONSTANT_DRIVER_KEY[0] && key[1] == CONSTANT_DRIVER_KEY[1] && key[2] == CONSTANT_DRIVER_KEY[2]) {
        // printf("Task builder CONSTANT_DRIVER_KEY\n");
        return new ConstTask();
    }
    else if (key[0] == SINUSIOD_DRIVER_KEY[0] && key[1] == SINUSIOD_DRIVER_KEY[1] && key[2] == SINUSIOD_DRIVER_KEY[2]) {
        // printf("Task builder SINUSIOD_DRIVER_KEY\n");
        return new SinTask();
    }
    else {
        // printf("Task builder\n");
        return new Task();
    }
}