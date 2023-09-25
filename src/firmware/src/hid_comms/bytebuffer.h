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

#ifndef BYTEBUFFER_H
#define BYTEBUFFER_H

#include <Arduino.h>
#include "utilities/blink.h"

typedef union
{
	float number;
	byte bytes[4];
} FLOATBYTE_t;

/*
	buffer of n bytes
	read and write its data.
*/

template <int buffer_size> class ByteBuffer {
	private:
		float timestamp;
		byte data[buffer_size];

	public:

		void print(){
			/*
				  Display function for HID packets
			*/
			printf("\n\tByteBuffer =====\n");
			if (buffer_size > 16) {
				for (int i = 0; i < buffer_size - 15; i += 16){
					printf("\t[%d]\t\t%X, %X, %X, %X, %X, %X, %X, %X, %X, %X, %X, %X, %X, %X, %X, %X\n", 
								i,
								data[i], data[i+1], data[i+2], data[i+3],
								data[i+4], data[i+5], data[i+6], data[i+7], 
								data[i+8], data[i+9], data[i+10], data[i+11],
								data[i+12], data[i+13], data[i+14], data[i+15]);
				}
			}
			else {
				printf("\t\t\n");
				for (int i = 0; i < buffer_size; i++) {
					printf("%X, ", data[i]);
				}
				printf("\n");
			}
		}

		// clear all data in the packet (zero)
		void clear() {
			memset(data, 0, buffer_size);
		}

		// Getters/Setters
		byte* buffer() {
			return data;
		}

		template <typename T> void put(int index, T value) {
			if (index + sizeof(T) <= buffer_size) {
				memcpy(&data[index], &value, sizeof(T));
			}
		}

		template <typename T> void put(int index, int n, T* values){
			for (int i = 0; i < n; i++) {
				put(index + (i * sizeof(T)), values[i]);
			}
			// memcpy(&data[index], values, n * sizeof(T));
		}

		template <typename T> T get(int index) {
			if (index + sizeof(T) <= buffer_size) {
				T value;
				memcpy(&value, &data[index], sizeof(T));
				return value;
			}
			return 0;
		}

		template <typename T> void get(int index, int n, T* buffer) {
			for (int i = 0; i < n; i++) {
				buffer[i] = get<T>(index + (i * sizeof(T)));
			}
			// memcpy(&data[index], buffer, n *sizeof(T));
		}
};

#endif