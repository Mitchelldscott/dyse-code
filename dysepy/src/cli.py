#!/usr/bin/env python3

import os
import sys
import yaml
import glob
import shutil
import argparse
import subprocess
from dysepy_tools import *
from robot_installer import *
from build_profile import Build_Profile
from uml_generator import UML_Generator

# This is DysePy

def clean_profile(profile):
	bp = Build_Profile();
	bp.load_profile(profile)
	bp.run_clean()

def build_profile(profile):
	bp = Build_Profile();
	bp.load_profile(profile)
	bp.run_build()

def main():
	parser = argparse.ArgumentParser(prog=sys.argv[0], description='CURO CLI Toolset')
	parser.add_argument('-b', '--build', 
		nargs='?',
		metavar='Profile',
		const='',
		help='Builds the workspace locally')
	parser.add_argument('--train', 
		metavar='Model',
		default='',
		help='Trains a Yolov5 model locally')
	parser.add_argument('-d', '--deploy', 
		action='store_true',
		help='Deploys build to the registered robots')
	parser.add_argument('-i', '--initialize', 
		action='store_true',
		help='Initializes registered devices')
	parser.add_argument('-c', '--clean',
		nargs='?',
		metavar='Profile',
		const='',
		help='Clean the entire workspace or a project. use profile=(lib, data) to clean workspace')
	parser.add_argument('-g', '--graph',
		nargs='+',
		metavar=['Profile', 'subdir'],
		help='Generate a drawio diagram of source code or fw tasks')

	ap = parser.parse_args(sys.argv[1:])

	if ap.clean:
		if ap.clean in DysePy_LOC_LUT: # clean the workspace
			reset_directory(DysePy_LOC_LUT[ap.clean])
			dyse_log(f"Reset directory {DysePy_LOC_LUT[ap.clean]}", 0)
		else:	# clean a profile
			clean_profile(ap.clean)

	if ap.build:
		build_profile(ap.build)

	if ap.initialize:
		initialize_devices()

	if ap.deploy:
		deploy_all_devices()

	if ap.graph:
		bp = Build_Profile();
		if bp.is_profile(ap.graph[0]):
			if bp.load_profile(ap.graph[0]):

				if len(ap.graph) > 1:
					graph_gen = UML_Generator(bp.project_path(), DysePy_LOC_LUT['docs'], ap.graph[1])
				else:
					graph_gen = UML_Generator(bp.project_path(), DysePy_LOC_LUT['docs'])

		else:
			graph_gen = UML_Generator(ap.graph[0], DysePy_LOC_LUT['docs'])
	
		graph_gen.generate()


if __name__ == '__main__':
	main()