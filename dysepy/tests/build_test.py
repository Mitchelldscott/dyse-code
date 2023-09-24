#!/usr/bin/env python3

import os
from build_profile import Build_Profile

demo_profile = os.path.join(os.getenv("PROJECT_ROOT"), "dysepy", "data", "build", "template.yaml")
dysepy_src = os.path.join(os.getenv("PROJECT_ROOT"), "src")

def main():
	bp = Build_Profile();
	bp.load_profile(demo_profile, dysepy_src)
	bp.run_build()
	bp.run_clean()

if __name__ == '__main__':
	main()