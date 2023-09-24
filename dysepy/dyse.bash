#! /bin/bash

#		Setup robot params
export SUDO='sudo'
export DOCKER=False
export PLATFORM=$(uname)
export PROJECT_ROOT=${PWD}
export DYSE_CORE_URI="127.0.0.1:1313"

#################### Platform specific setup ####################

if [[ "$PLATFORM" == "Linux"* ]]; then
	export UBUNTU_VERSION=$(cut -f2 <<< $(lsb_release -r))
elif [[ "$PLATFORM" == "Darwin" || "$PLATFORM" == "msys" || "$PLATFORM" == "win32" ]]; then
	export UBUNTU_VERSION="False"
else
	echo -e "Unsupported OS... good luck!"
fi

echo -e "Welcome to $(basename ${PROJECT_ROOT}), ${USER}"
echo -e "\tSetting up tools for ${PLATFORM}"
echo -e "\tUbuntu\tDocker\tProject Root"
echo -e "\t${UBUNTU_VERSION}\t${DOCKER}\t${PROJECT_ROOT}"
git status

#################### Docker tools setup ####################

if [[ -f /.dockerenv ]]; then
	SUDO=''
	DOCKER=True
	PROJECT_ROOT=/home/dyse-robotics/dyse-code
fi

if [[ "${UBUNTU_VERSION}" == "20.04" ]]; then
	export ROS_DISTRO=noetic				# ROS for Ubuntu18
elif [[ "${UBUNTU_VERSION}" == "18.04" ]]; then
	export ROS_DISTRO=melodic
else
	export ROS_DISTRO=
fi


if [[ "${DOCKER}" == "False" ]]; then
	alias spinup="cd ${PROJECT_ROOT}/containers && \
		docker compose run "
else
	export LC_ALL=C.UTF-8
    export LANG=C.UTF-8
fi

if [[ "${PROJECT_ROOT}" != */rufous ]]; then
	echo -e "running from ${PWD}, is this your project root?"
	return
fi

#################### Python/ROS tools setup ####################

PYTHONPATH=

# If ROS is installed source the setup file

if [[ -f /opt/ros/${ROS_DISTRO}/setup.bash ]]; then
	source /opt/ros/${ROS_DISTRO}/setup.bash
fi

# set ROS package path to buff-code so it can see dysepy
if [[ "${ROS_PACKAGE_PATH}" != *"rufous"* ]]; then
	export ROS_PACKAGE_PATH="${PROJECT_ROOT}:${ROS_PACKAGE_PATH}"
fi

if [[ "$1" == "reset" ]]; then
	if [[ -f "/usr/local/bin/dysepy" ]]; then
		${SUDO} rm -rf "/usr/local/bin/dysepy"
	fi
	if [[ -f "/usr/local/bin/run" ]]; then
		${SUDO} rm -rf "/usr/local/bin/run"
	fi
fi

if [[ ! -f "/usr/local/bin/dysepy" ]]; then
	${SUDO} touch "/usr/local/bin/dysepy"
	echo "/usr/bin/env python3 \${PROJECT_ROOT}/dysepy/src/cli.py \$@" | ${SUDO} tee "/usr/local/bin/dysepy"
	${SUDO} chmod +x "/usr/local/bin/dysepy"
fi 

if [[ ! -f "/usr/local/bin/run" ]]; then
	${SUDO} touch "/usr/local/bin/run"
	echo "/usr/bin/env python3 \${PROJECT_ROOT}/dysepy/src/robot_spawner.py \$@" | ${SUDO} tee "/usr/local/bin/run"
	${SUDO} chmod +x "/usr/local/bin/run"
fi 

# Only export if if not already in path
if [[ "${PYTHONPATH}" != *"${PROJECT_ROOT}/dysepy/lib:"* ]]; then	
	export PYTHONPATH="${PROJECT_ROOT}/dysepy/lib:${PYTHONPATH}" 
fi

if [[ "${PYTHONPATH}" != *"${PROJECT_ROOT}/dysepy/src:"* ]]; then	
	export PYTHONPATH="${PROJECT_ROOT}/dysepy/src:${PYTHONPATH}" 
fi

#################### Cargo setup ####################

if [[ "${PATH}" != *"/.cargo"*  && -f ${HOME}/.cargo/env ]]; then
	source ${HOME}/.cargo/env
fi

#################### Bash tools setup ####################

alias bc="cd ${PROJECT_ROOT}"
alias br="cd ${PROJECT_ROOT}/src/buff_rust"
alias fw="cd ${PROJECT_ROOT}/src/firmware"
alias bn="cd ${PROJECT_ROOT}/src/rknn_buffnet"

# Not totally clear but this solves an 
# illegal instruction error with rospy.
# Only for Jetson
# the status of this issue needs to be double checked
if [[ "${HOSTNAME}" == "edge"* ]]; then
	export OPENBLAS_CORETYPE=ARMV8
	export ROS_IP=$(/sbin/ip -o -4 addr list wlan0 | awk '{print $4}' | cut -d/ -f1)
	export ROS_MASTER_URI=http://${ROS_IP}:11311
else
	# if not on jetson set the user IP
	# should figure out how to set it if it is on the jetson
	# export USER_IP=$(/sbin/ip -o -4 addr list wlp3s0 | awk '{print $4}' | cut -d/ -f1) # Needs testing

	alias buildr="dysepy -b rust-debug"
	alias buildf="dysepy -b fw"
	alias builda="dysepy -b all"
	alias buff-test="br && cargo test"
	alias sshbot="ssh -X cu-robotics@edgek.local"
	alias scp-src="scp -r ~/buff-code/src/rknn_buffnet/src cu-robotics@edgek.local:/home/dyse-robotics/dyse-code/src/rknn_buffnet"
	alias scp-h="scp -r ~/buff-code/src/rknn_buffnet/include cu-robotics@edgek.local:/home/dyse-robotics/dyse-code/src/rknn_buffnet"
	set-ros-master () {
		export ROS_MASTER_URI=http://$1:11311
	}
fi

