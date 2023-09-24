#!/usr/bin/python3
import os
import sys
import yaml
import time
import numpy as np
from scipy import linalg as la
import matplotlib.pyplot as plt

import rospy
from std_msgs.msg import Float64MultiArray

collect_samples = False
control = []
pin_output = []
teensy_time = []

def callback(msg):
	if collect_samples:
		pin_output.append(msg.data[0]);
		teensy_time.append(msg.data[1]);

def fb_callback(msg, pub):
	if collect_samples:
		pin_output.append(msg.data[0]);
		teensy_time.append(msg.data[1]);
		control.append(((pin_output[-1] / 4096) + 0.01) % 1);
		msg = Float64MultiArray()
		msg.data = [control[-1]]
		pub.publish(msg)


def discrete_impulse(magnitude=0.1, length=100, start=10, stop=25):
	control = []
	for i in range(length):
		if i >= start and i < stop:
			control.append(magnitude)
		else:
			control.append(0)

	return control

def discrete_impulses(magnitude=0.05, length=100):
	ctr = 0
	sign = 1
	control = []
	for i in range(length):
		if ctr > length / 10 :
			ctr = 0
			mag = magnitude * sign
			sign *= -1
		else:
			mag = 0.0

		control.append(mag)
		ctr += 1

	return control

def sinusiod(length=100, start=10, stop=60):
	control = []
	amplitudes = 1
	frequencies = 0.1

	for i in range(length):
		if i >= start and i < stop:
			control.append(np.sum(amplitudes * np.sin(frequencies * i)))
		else:
			control.append(0)

	return control

def freq_sweep_sinusiod(length=100, start=10, stop=60):
	control = []
	amplitudes = np.linspace(0, 10, 150) / 500
	frequencies = np.linspace(0, 100, 150)

	for i in range(length):
		if i >= start and i < stop:
			control.append(np.sum(amplitudes * np.sin(frequencies * i)))
		else:
			control.append(0)

	return control

def send_inputs(hz, control, target_pin):
	global collect_samples
	global pin_output
	global teensy_time

	pub = rospy.Publisher(f"signal_generator_ctrl", Float64MultiArray, queue_size=10)
	rate = rospy.Rate(hz)
	msg = Float64MultiArray()

	rospy.Subscriber(f"led_output{target_pin}", Float64MultiArray, callback)

	print("Collection Start")

	collect_samples = True
	
	for u in control:

		msg.data = [u]
		pub.publish(msg)
		rate.sleep()

		if rospy.is_shutdown():
			exit(0)

	collect_samples = False
	msg.data = np.zeros(1) # shutdown
	pub.publish(msg)

	print("Data Collection Finished")

	if len(pin_output) == 0:
		print("No samples exiting")
		exit(0)

	print(f"Duration: {duration} secs")
	print(f"Teensy Start/Stop: {teensy_time[-1]} {teensy_time[0]}")
	print(f"Teensy Duration: {(teensy_time[-1] - teensy_time[0])}")
	print(f"Teensy Rate: {1 / np.mean(np.array(teensy_time[1:]) - np.array(teensy_time[:-1]))}")
	print(f"Feedback packets: {np.array(pin_output).shape}")
	print(f"Feedback timestamps: {np.array(teensy_time).shape}")

	return (control, pin_output, np.array(teensy_time) - teensy_time[0])


def spin_feedback(hz, duration, target_pin):
	global collect_samples
	global pin_output
	global teensy_time
	global control

	pub = rospy.Publisher(f"led{target_pin}_ictrl", Float64MultiArray, queue_size=10)
	rate = rospy.Rate(hz)
	msg = Float64MultiArray()

	rospy.Subscriber(f"led{target_pin}", Float64MultiArray, fb_callback, pub)

	print("Collection Start")

	collect_samples = True
	
	i = 0
	while i < duration:
		rate.sleep()
		i += 1 / hz
		if rospy.is_shutdown():
			exit(0)

	collect_samples = False
	msg.data = np.zeros(1) # shutdown
	pub.publish(msg)

	print("Data Collection Finished")

	if len(pin_output) == 0:
		print("No samples exiting")
		exit(0)


	print(f"Pin Feedback test {target_pin} ==========")
	print(f"\tDuration: {duration} secs")
	print(f"\tTeensy Start/Stop: {teensy_time[-1]} {teensy_time[0]}")
	print(f"\tTeensy Duration: {(teensy_time[-1] - teensy_time[0])}")
	print(f"\tTeensy Rate: {1/np.mean(np.array(teensy_time[1:]) - np.array(teensy_time[:-1]))}")
	print(f"\tFeedback packets: {np.array(pin_output).shape}")
	print(f"\tFeedback timestamps: {np.array(teensy_time).shape}")
	print(f"\tControl packets: {np.array(teensy_time).shape}")
	print(f"\tFeedback slope {pin_output[-1] / (teensy_time[-1] - teensy_time[0])}")
	print(f"\tControl slope {control[-1] / duration}")

	return (control, pin_output, np.array(teensy_time) - teensy_time[0])

# def find_edge(arr, threashold):
# 	prev = arr[0]

# 	for (i, x) in enumerate(arr[1:]):
# 		if abs(x - prev) > threashold:
# 			return i+1

# 	return -1

# def find_control_delay(duration, control):
# 	ctrl_edge = find_edge(control, 0.001)
# 	pos_edge = find_edge(pin_output, 0.001)

# 	"""
# 		(ctrl_edge / ctrl_len) + delay = (pos_edge / pos_len)
# 		delay = (pos_edge / pos_len) - (ctrl_edge / ctrl_len)
# 	"""
# 	# teensy_shift = 
# 	control_shift = (out_edge / len(pin_output)) - (ctrl_edge / len(control))


# 	print(f"Control to Output latency: {control_shift * duration}")
	
# 	return control_shift

def display_data(pin, duration, control, outputs, output_time):
	# control_shift = int(find_control_delay(duration, control) * duration)

	ctrl_steps = np.linspace(0, duration, num=len(control))

	time_ratio = duration / output_time[-1]
	# fig, axes = plt.subplots(2, 2, figsize=(8,15))
	plt.title(f"Signal and feedback for {pin}")
	plt.plot(ctrl_steps, 4096 * np.array(control), label="Generated signal", color='r', marker='o')
	plt.plot(output_time * time_ratio, outputs, label="hardware feeback", color='b', marker='x')
	plt.legend()
	plt.show()

if __name__ == '__main__':
	try:
		rospy.init_node('delay_test', anonymous=True)

		target_pin = 1
		if len(sys.argv) > 1:
			target_pin = sys.argv[1]

		time.sleep(1)

		# control = np.abs(np.array(sinusiod(start=10, stop=80))) * 50
		# control = discrete_impulse(magnitude=255, length=50)
		# control = np.abs(discrete_impulses(magnitude=255, length=50))

		print(f"Starting system test for pin {target_pin}")

		# hz = 10
		# duration = len(control) / hz
		# (control, outputs, output_time) = send_inputs(hz, control, target_pin)
		
		hz = 10
		duration = 5
		(control, outputs, output_time) = spin_feedback(hz, duration, target_pin)
		
		display_data(target_pin, duration, control, outputs, output_time)

	except rospy.ROSInterruptException:
		pass