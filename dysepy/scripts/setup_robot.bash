#! /bin/bash

sudo cp /home/dyse-robotics/dyse-code/dysepy/scripts/dysebot.service /etc/systemd/system

sudo systemctl enable dysebot.service
