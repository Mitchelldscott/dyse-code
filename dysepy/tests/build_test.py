#!/usr/bin/env python3

import os
from build_profile import Build_Profile

demo_profile = "template"

def main():
	bp = Build_Profile();
	bp.load_profile(demo_profile)
	bp.run_build()
	bp.run_clean()

if __name__ == '__main__':
	main()