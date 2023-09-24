#! /bin/bash

#!/bin/bash
set -e

buffbash="/home/dyse-robotics/dyse-code/dysepy/dyse.bash"
echo "sourcing   	$buffbash"
source "$buffbash"

echo "Host 		$HOSTNAME"
echo "Project Root 	$PROJECT_ROOT"

exec "$@"