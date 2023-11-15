#!/usr/bin/env python3

import os
import sys
import yaml
import subprocess as sb

from dysepy_tools import *

class Build_Profile:
	"""
		Build Profiles are a core feature of the dysepy development process.
		A build profile will tell dysepy how to build, clean and install a
		project. This implementation is meant to scale to any number, type and size
		of project.

		A project is a directory under dyse-code/src that makes use of a
		build system or a program that will generate files/binaries for
		the robot's software.

		TODO: (lots of room to grow)
			- add project tracking
			  - this could look like a cache file the build profile generates
			  		when building or cleaning, the file describes the status
			  		(don't rebuild when you don't need to)
			- integrate with run system
			  - This will allow run to build the necesary projects making
			  		building obsolete, all you do is run the sw (one command does it all)

	"""
	def __init__(self):
		self.name = None			# Name/dir of the project (relative to self.path)
		self.profile = None			# full path to profile, includes filename (absolute)
		self.setup_cmd = None		# Command to initialize the project, runs at path (ie 'cd self.path && setup_cmd')
		self.build_cmd = None		# Command to build the project (and install) runs at project_path()
		self.clean_cmd = None		# Command to clean the project same run location as build
		self.target_src = []		# Paths to files generated by build process (executables, binaries, etc) (relative to self.project_path())
		self.target_dst = []		# Paths to install files to, may be keywords
		self.includes = []

	def project_path(self):
		"""
			Get the absolute path to the project
			combines self.path and self.name 
		"""
		if self.name is None:
			return ""

		return os.path.join(DysePy_LOC_LUT['src'], self.name)

	def is_profile(self, profile_path):
		if os.path.exists(profile_path) and profile_path.split('/')[-2] == 'build':
			return True

		return False


	def load_profile(self, profile):
		"""
			Parse a yaml file into a build profile requires a valid yaml file and project
			setup. Both paths must be absolute and valid

			@param:
				profile: name of profile without .yaml extension
		"""
		# Get full path
		profile_path = os.path.join(DysePy_LOC_LUT['profiles'], f'{profile}.yaml')

		if not self.is_profile(profile_path):
			dyse_log(f"Profile {profile_path} does not exist", 2)
			return False

		with open(profile_path, 'r') as profile:
			contents = yaml.safe_load(profile)

		self.profile = profile_path

		if 'project' in contents:
			self.name = contents['project']
		else:
			self.name = ""

		if 'setup' in contents:
			self.setup_cmd = contents['setup']

		if 'build' in contents:
			self.build_cmd = contents['build']

		if 'clean' in contents:
			self.clean_cmd = contents['clean']

		if 'targets' in contents:
			self.target_src = contents['targets']

		if 'install' in contents:
			self.target_dst = contents['install']

		if 'include' in contents:
			for profile in contents['include']:
				self.includes.append(Build_Profile())
				self.includes[-1].load_profile(profile)

		return True

	def assert_setup(self):
		"""
			Some profiles require a setup (cloning from git or just creating some files)
			Runs the setup_cmd from dyse-code/src or whatever the project dir is.
		"""
		if self.name == "":
			dyse_log(f"Virtual Project detected", 0)
			return 2

		if not os.path.exists(self.project_path()):
			dyse_log(f"Can't find project: attempting setup ...", 1)
			if self.run_setup():
				dyse_log(f"No existing setup for {self.name}", 2)
				return 1

		return 0

	def validate_targets(self):
		"""
			Check if number of targets and install locations match
		"""
		if len(self.target_dst) != len(self.target_src):
			dyse_log(f"Target and install mismatch {len(self.target_src)} != {len(self.target_dst)}", 2)
			return 1

		return 0

	def validate_build(self):
		"""
			Check if the build was successful
			Only depends on existence of generated files
			  if the project was already built it will pass.
		"""
		status = False
		for target in self.target_src:
			status |= not os.path.exists(os.path.join(self.project_path(), target))

		dyse_log(f"{self.name} Built", 2 * status) # error if True info if False

	def validate_install(self):
		"""
			Check if the install was successful
			Only checks if installed files exist
			  if the project was already installed it will pass
		"""
		status = False
		for (src, dst) in zip(self.target_src, [DysePy_LOC_LUT[dst] for dst in self.target_dst]):
			sb.run(['chmod', '+x', os.path.join(dst, src.split('/')[-1])])
			status |= not os.path.exists(os.path.join(dst, src.split('/')[-1]))

		dyse_log(f"{self.name} Installed", 2 * status) # error if True info if False

	def validate_clean(self):
		"""
			Check if the clean was successful
			Only checks that the generated files do not exist
		"""
		status = False
		for target in self.target_src:
			status |= os.path.exists(os.path.join(self.project_path(), target))

		dyse_log(f"{self.name} Cleaned", 2 * status) # error if True info if False

	def run_job(self, base_cmd, path):
		"""
			Convenience function for executing jobs, if the base_cmd or profile
			  is None, there is no job to do.

			@params
		"""
		if self.name is None:
			dyse_log(f"Name {self.name} is invalid", 2)
			return None

		if not base_cmd is None:
			cmd = f'cd {path} && {base_cmd}'
			sb.run(cmd, shell=True)

	def run_setup(self):
		"""
			Run the setup command from project dir (dyse-code/src)
			@returns
				status of setup
		"""
		self.run_job(self.setup_cmd, DysePy_LOC_LUT['src'])
		dyse_log(f"Finished {self.name} setup", not os.path.exists(self.project_path()))
		return not os.path.exists(self.project_path())

	def run_build(self):
		"""
			Run the build command from project root (dyse-code/src/PROJECT_NAME)
		"""
		if self.assert_setup() == 0:
			if self.validate_targets():
				return

			dyse_log(f"Building {self.name}", 0)
			self.run_job(self.build_cmd, self.project_path())
			self.validate_build()

			dyse_log(f"Installing {self.name}", 0)
			copy_packages(self.project_path(), self.target_src, [DysePy_LOC_LUT[dst] for dst in self.target_dst])
			self.validate_install()

		for include in self.includes:
			include.run_build()

	def run_clean(self):
		"""
			run the clean command from project root (dyse-code/src/PROJECT_NAME)
		"""
		if self.name != "":
			if not os.path.exists(self.project_path()):
				dyse_log(f"Can't find project: nothing to clean", 1)
				return

			dyse_log(f"Cleaning {self.name}", 0)
			self.run_job(self.clean_cmd, self.project_path())
			self.validate_clean()

		for include in self.includes:
			include.run_clean()

	def dump_info(self):
		print(f"{self.name} profile description")
		print(f"\tprofile:\t{self.profile}")
		print(f"\tproject:\t{DysePy_LOC_LUT['src']}")
		print(f"\tsetup:\t{self.setup_cmd}")
		print(f"\tbuild:\t{self.build_cmd}")
		print(f"\tclean:\t{self.clean_cmd}")


