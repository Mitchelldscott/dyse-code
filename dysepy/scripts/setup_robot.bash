#! /bin/bash

sudo cp /home/dyse-robotics/dyse-code/dysepy/scripts/buffbot.service /etc/systemd/system

sudo systemctl enable buffbot.service
