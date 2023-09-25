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

#include "hid_comms.h"
#include "utilities/assertions.h"

float hid_errors = 0;

float pc_lifetime = 0;
float pc_read_count = 0;
float pc_write_count = 0;
float mcu_read_count = 0;
float mcu_write_count = 0;

FTYK hid_timers;
ByteBuffer<64> buffer;
IntervalTimer hid_interval_timer;

CommsPipeline pipeline;

void push_hid() {

	if(!usb_rawhid_available()) {
		// printf("RawHID not available %f\n", hid_timers.secs(1));
		if (hid_timers.secs(0) > 5) { // reset stats when no connection
			reset_hid_stats();
			hid_timers.set(0);
		}
		// printf("USB not available\n");
		// clear_feedback_pipeline();
		return;
	}

	hid_timers.set(0);

	switch (usb_rawhid_recv(buffer.buffer(), 0)) {
		case 64:
			blink();										// only blink when connected to hid
			mcu_read_count += 1;
			// printf("Recieved Report %i, %i\n", buffer.get<byte>(0), buffer.get<byte>(1));

			switch (buffer.get<byte>(0)) {
				case 255:
					switch (buffer.get<byte>(1)) {
						case 1:
							init_task_hid();
							send_hid_status();
							return;

						case 2:
							config_task_hid();
							send_hid_status();
							return;

						default:
							break;
					}
					break;

				case 1:
					overwrite_task_hid();
					break;

				case 13:
					printf("kill switch received\n");
					send_hid_status();
					reset_hid_stats();
					return;

				default:
					break;
			}
			break;
		
		default:
			printf("No packet available\n");
			// clear_feedback_pipeline();
			break;
	}

	// while (pipeline.feedback.size() > 0) {
	// 	// printf("Feedback queue: %i %i\n", pipeline.feedback.size(), pipeline.feedback.len());
	// 	send_hid_feedback();
	// 	clear_feedback_pipeline();	
	// }

	if (pipeline.feedback.size() > 0) {
		send_hid_feedback();
	}
	else {
		send_hid_status();
	}
}

// void clear_feedback_pipeline() {
// 	int fb_size = pipeline.feedback.size();
// 	for (int i = 0; i < fb_size; i++) {
// 		TaskFeedback* fb = pipeline.feedback.pop();
// 		if (fb) {
// 			delete fb;
// 		}
// 	}
// 	pipeline.feedback.reset(0);
// }

void send_hid_status() {

	buffer.put<byte>(0, 255);
	buffer.put<byte>(1, 255);
	buffer.put<float>(2, mcu_write_count);
	buffer.put<float>(6, mcu_read_count);
	send_hid_with_timestamp();
}

void send_hid_feedback() {
	static int task_num = 0;

	for (int i = 0; i < pipeline.feedback.size(); i++) {
		if (pipeline.feedback[task_num]->configured == 0) {
			break;
		}
		if (pipeline.feedback[task_num]->update <= 0 || pipeline.feedback[task_num]->output.size() <= 0) {
			task_num = (task_num + 1) % pipeline.feedback.size();
		}
		else {
			pipeline.feedback[task_num]->update = 0;
			break;
		}
	}

	// printf("pipeline feedback: %p %i\n", pipeline.feedback[task_num], pipeline.feedback[task_num]->task_id);
	buffer.put<byte>(0, 1);
	buffer.put<byte>(1, pipeline.feedback[task_num]->task_id);
	buffer.put<byte>(2, pipeline.feedback[task_num]->latch);
	dump_vector(&pipeline.feedback[task_num]->output);
	buffer.put<float>(56, pipeline.feedback[task_num]->timestamp);
	task_num = (task_num + 1) % pipeline.feedback.size();
	send_hid_with_timestamp();
}

void send_hid_with_timestamp() {
	pc_lifetime = buffer.get<float>(60);
	buffer.put<float>(60, pipeline.lifetime);
	if (usb_rawhid_send(buffer.buffer(), 0) > 0) {
		mcu_write_count += 1;
	}
	else {
		printf("failed to write\n");
	}
}

void init_task_hid() {

	TaskSetupPacket* task = new TaskSetupPacket;
	
	task->packet_type = 0;
	task->task_id = buffer.get<byte>(2);
	task->rate = buffer.get<uint16_t>(3);
	task->key[0] = buffer.get<char>(5);
	task->key[1] = buffer.get<char>(6);
	task->key[2] = buffer.get<char>(7);
	task->n_inputs = buffer.get<byte>(10);
	task->inputs.reset(task->n_inputs);

	// printf("Init task %i %i %i %c%c%c\n", task->task_id, task->n_inputs, task->rate, task->key[0], task->key[1], task->key[2]);
	for (int i = 0; i < task->n_inputs; i++) {
		task->inputs[i] = buffer.get<byte>(11 + i);
	}

	// push to setup queue
	pipeline.setup_queue.push(task);
}

void config_task_hid() {

	TaskSetupPacket* task = new TaskSetupPacket;

	task->packet_type = 1;
	task->task_id = buffer.get<byte>(2);
	task->chunk_id = buffer.get<byte>(3);
	task->chunk_size = buffer.get<byte>(4);
	task->parameters.reset(task->chunk_size);

	// printf("Config task %i %i %i\n", task->task_id, task->chunk_id, task->chunk_size);
	for (int i = 0; i < task->chunk_size; i++) {
		task->parameters[i] = buffer.get<float>((4 * i) + 5);
	}

	// push config packet to setup queue
	pipeline.setup_queue.push(task);
}

void overwrite_task_hid() {

	TaskSetupPacket* task = new TaskSetupPacket;

	task->packet_type = 2;
	task->task_id = buffer.get<byte>(1);
	task->latch = buffer.get<byte>(2);
	task->data_len = buffer.get<byte>(3);
	task->data.reset(task->data_len);

	// printf("Overwrite task %i %i %i\n", task->task_id, task->latch, task->data_len);
	for (int i = 0; i < task->data_len; i++) {
		task->data[i] = buffer.get<float>((4 * i) + 4);
	}

	// push config packet to setup queue
	pipeline.setup_queue.push(task);
}

void reset_hid_stats() {
	mcu_write_count = 0;
	mcu_read_count = 0;
	pc_write_count = 0;
	pc_read_count = 0;
	pc_lifetime = 0;
	hid_errors = 0;

	pipeline.lifetime = 0;
	// clear_feedback_pipeline();
}

void dump_vector(Vector<float>* data) {
	buffer.put<byte>(3, data->size());
	buffer.put<float>(4, data->size(), data->as_array());
}

CommsPipeline* enable_hid_interrupts() {
	pipeline.feedback.reset(0);
	pipeline.setup_queue.reset(0);
	hid_interval_timer.begin(push_hid, HID_REFRESH_RATE);
	return &pipeline;
}

template <> void Vector<TaskSetupPacket*>::print() {
	if (length == 0) {
		printf("Task Setup Vector [empty]\n");
		return;
	}

	printf("Task Setup Vector [%i]: [\n", length);
	for (int i = 0; i < length; i++) {
		buffer[i]->print();
	}
	printf("]\n");
}