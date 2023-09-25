#include <SPI.h>  // include the SPI library:
#include "utilities/blink.h"
#include "utilities/splash.h"
#include "utilities/assertions.h"

#define LSM9DS1_REGISTER_CTRL_REG8 0x22
#define LSM9DS1_XG_ID (0b01101000)
#define LSM9DS1_REGISTER_WHO_AM_I_XG 0x0F
#define LSM9DS1_SEND_VALUE 0xFF

const int chip_select = 37;

int readRegister(int cs, byte reg, int n_bytes) {
  int result = 0;   // result to return

  // gain control of the SPI port
  // and configure settings
  // SPI1.beginTransaction(SPISettings(1000000, MSBFIRST, SPI_MODE1));
  // take the chip select low to select the device:
  digitalWrite(cs, LOW);
  // send the device the register you want to read:
  printf("initial send: %i\n", SPI1.transfer(reg | 0x80));

  result = SPI1.transfer(LSM9DS1_SEND_VALUE);

  // take the chip select high to de-select:
  digitalWrite(cs, HIGH);
  // release control of the SPI port
  // SPI1.endTransaction();
  return result;
}


//Sends a write command to SCP1000

void writeRegister(int cs, byte reg, byte value) {

  // gain control of the SPI port
  // and configure settings
  // SPI1.beginTransaction(SPISettings(1000000, MSBFIRST, SPI_MODE1));
  // take the chip select low to select the device:
  digitalWrite(cs, LOW);

  SPI1.transfer(reg & 0x3F);
  SPI1.transfer(value);

  // take the chip select high to de-select:
  digitalWrite(cs, HIGH);
  // release control of the SPI port
  // SPI1.endTransaction();
}


int meet_device() {
	
	int value = readRegister(chip_select, 0x0F, 1); 

	assert_eq<int>(value, LSM9DS1_XG_ID, "cannot find target device");

	return value;
}

void setup() {

	setup_blink();
	// set the chip_select as an output:
	pinMode(chip_select, OUTPUT);

	// initialize SPI:
	SPI1.begin(); 

	writeRegister(chip_select, LSM9DS1_REGISTER_CTRL_REG8, 0x05);
	delay(10);
}

int main() {

	unit_test_splash("spi driver", -1);
	setup();

	while (1) {
		if (meet_device())
			blink();

		delay(250);
	}

	return 0;
}