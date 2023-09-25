#include <Wire.h>
#include <SPI.h>
#include <Adafruit_LSM9DS1.h>
#include <Adafruit_Sensor.h>  // not used in this demo but required!
#include "utilities/timing.h"
#include "utilities/splash.h"
// i2c
// Adafruit_LSM9DS1 lsm = Adafruit_LSM9DS1();

#define LSM9DS1_MOSI    26
#define LSM9DS1_SCK     27
#define LSM9DS1_XGCS    37
#define LSM9DS1_MCS     38
#define LSM9DS1_MISO    39

FTYK timers;

// You can also use software SPI
Adafruit_LSM9DS1 lsm = Adafruit_LSM9DS1(LSM9DS1_SCK, LSM9DS1_MISO, LSM9DS1_MOSI, LSM9DS1_XGCS, LSM9DS1_MCS);
// Or hardware SPI! In this case, only CS pins are passed in
// Adafruit_LSM9DS1 lsm = Adafruit_LSM9DS1(LSM9DS1_XGCS, LSM9DS1_MCS);


void setupSensor()
{
  // 1.) Set the accelerometer range
  lsm.setupAccel(lsm.LSM9DS1_ACCELRANGE_16G);
  //lsm.setupAccel(lsm.LSM9DS1_ACCELRANGE_4G);
  //lsm.setupAccel(lsm.LSM9DS1_ACCELRANGE_8G);
  //lsm.setupAccel(lsm.LSM9DS1_ACCELRANGE_16G);
  
  // 2.) Set the magnetometer sensitivity
  lsm.setupMag(lsm.LSM9DS1_MAGGAIN_8GAUSS);
  //lsm.setupMag(lsm.LSM9DS1_MAGGAIN_8GAUSS);
  //lsm.setupMag(lsm.LSM9DS1_MAGGAIN_12GAUSS);
  //lsm.setupMag(lsm.LSM9DS1_MAGGAIN_16GAUSS);

  // 3.) Setup the gyroscope
  lsm.setupGyro(lsm.LSM9DS1_GYROSCALE_2000DPS);
  //lsm.setupGyro(lsm.LSM9DS1_GYROSCALE_500DPS);
  //lsm.setupGyro(lsm.LSM9DS1_GYROSCALE_2000DPS);
}


void setup() 
{
  
  // Try to initialise and warn if we couldn't detect the chip
  if (!lsm.begin())
  {
    printf("Oops ... unable to initialize the LSM9DS1. Check your wiring!\n");
    while (1);
  }
  printf("Found LSM9DS1 9DOF\n");

  // helper to just set the default scaling we want, see above!
  timers.set(0);
  setupSensor();
  timers.print(0, "sensor setup");
}

int main() 
{

  unit_test_splash("LSM9DS1", -1);
  setup();

  while (1) {
    // timers.set(1);

    timers.set(0);
    lsm.read();  /* ask it to read in the data */ 
    timers.print(0, "sensor read");

    /* Get a new sensor event */ 
    // timers.set(0);
    sensors_event_t a, m, g, temp;
    lsm.getEvent(&a, &m, &g, &temp); 
    // timers.print(0, "sensor event access");

    // printf("Accel X: %f m/s^2\n", a.acceleration.x);
    // printf("\tY: %f m/s^2\n", a.acceleration.y);
    // printf("\tZ: %f m/s^2\n", a.acceleration.z);

    // printf("Mag X: %f uT\n", m.magnetic.x);
    // printf("\tY: %f uT\n", m.magnetic.y);  
    // printf("\tZ: %f uT\n", m.magnetic.z);  

    // printf("Gyro X: %f rad/s\n", g.gyro.x);
    // printf("\tY: %f rad/s\n", g.gyro.y);   
    // printf("\tZ: %f rad/s\n", g.gyro.z);   

    // timers.print(1, "loop");
    timers.delay_micros(0, 500);
  }
    
  return 0;
}