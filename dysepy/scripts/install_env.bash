#! /bin/bash

#
# 	First install system dependencies through apt
#

echo -e "\n\tInstalling DyseCode Dependencies...\n"

$SUDO xargs apt install -y --no-install-recommends <${PROJECT_ROOT}/dysepy/data/install/dependencies.txt

#
#	Install pip with get-pip
# 		the system pip (from apt) is custom and
#		we would rather use vanilla pip (basic).

echo -e "\n\tInstalling pip3\n"

curl https://bootstrap.pypa.io/get-pip.py -o get-pip.py

$SUDO python3 get-pip.py

rm get-pip.py


#
#	Setup virtual envirnment
#
if [[ ! -d ${PROJECT_ROOT}/dysepy/env ]]; then
	cd ${PROJECT_ROOT}/dysepy && python3 -m venv env
fi

dc && source ${PROJECT_ROOT}/dysepy/env/bin/activate

#
#	Install python requirements with pip3
#

echo -e "\n\tInstalling Dyse-Code Python3 requirements\n"

pip install -r ${PROJECT_ROOT}/dysepy/data/install/python3_requirements.txt
