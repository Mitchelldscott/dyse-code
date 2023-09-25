#include "bytebuffer.h"
#include "utilities/blink.h"
#include "utilities/timing.h"
#include "utilities/vector.h"

#ifndef HIDCOMMS_H
#define HIDCOMMS_H

#define HID_REFRESH_RATE 200.0
#define MAX_FLOAT_DATA_PER_SEND 13

extern float hid_errors;

struct TaskSetupPacket {
	int task_id;
	int packet_type;
	
	char key[3];
	int rate;
	int n_inputs;
	Vector<int> inputs;

	int chunk_id;
	int chunk_size;
	Vector<float> parameters;

	int latch;
	int data_len;
	Vector<float> data;

	void print() {
		printf("ID: %i\nPacket type: %i\n", task_id, packet_type);
	}
};

struct TaskFeedback {
	// int error;	// someone should figure this out
	int task_id;
	int update;
	int configured;
	int latch;
	float timestamp;
	float secs;
	float mins;
	float hrs;
	Vector<float> output;
};

struct CommsPipeline {
	float lifetime;
	int minutes;
	int hours;

	Vector<TaskFeedback*> feedback;
	Vector<TaskSetupPacket*> setup_queue;
};

void push_hid();
void clear_feedback_pipeline();
void send_hid_status();
void send_hid_feedback();
void send_hid_with_timestamp();
void init_task_hid();
void config_task_hid();
void overwrite_task_hid();
void reset_hid_stats();
void dump_vector(Vector<float>*);
CommsPipeline* enable_hid_interrupts();

#endif