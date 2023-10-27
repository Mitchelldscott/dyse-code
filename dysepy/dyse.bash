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

if [[ "${PROJECT_ROOT}" != */dyse-code ]]; then
	echo -e "running from ${PWD}, is this your project root?"
	return
fi

#################### Python tools setup ####################

PYTHONPATH=

# If ROS is installed source the setup file

# if [[ -f /opt/ros/${ROS_DISTRO}/setup.bash ]]; then
# 	source /opt/ros/${ROS_DISTRO}/setup.bash
# fi

# set ROS package path to dyse-code so it can see dysepy
# if [[ "${ROS_PACKAGE_PATH}" != *"rufous"* ]]; then
# 	export ROS_PACKAGE_PATH="${PROJECT_ROOT}:${ROS_PACKAGE_PATH}"
# fi

# This needs an update.
# install dysepy source to the env
# in dysepy, then share that env (no more echo cmd >> /usr/bin/file)
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

#################### Network setup ####################

${SUDO} ifconfig lo multicast
${SUDO} route add -net 224.0.0.0 netmask 240.0.0.0 dev lo


#################### Bash tools setup ####################

alias dc="cd ${PROJECT_ROOT}"
alias dr="cd ${PROJECT_ROOT}/src/dyse_rust"
alias fw="cd ${PROJECT_ROOT}/src/firmware"

# if [[ "${HOSTNAME}" == "edge"* ]]; then
# 	# export ROS_IP=$(/sbin/ip -o -4 addr list wlan0 | awk '{print $4}' | cut -d/ -f1)
# 	# export ROS_MASTER_URI=http://${ROS_IP}:11311
# else
# 	# if not on jetson set the user IP
# 	# should figure out how to set it if it is on the jetson
# 	# export USER_IP=$(/sbin/ip -o -4 addr list wlp3s0 | awk '{print $4}' | cut -d/ -f1) # Needs testing

# 	# alias bldr="dysepy -b rust-debug"
# 	# alias bldf="dysepy -b fw"
# 	# alias blda="dysepy -b all"
# 	# set-ros-master () {
# 	# 	export ROS_MASTER_URI=http://$1:11311
# 	# }
# fi

