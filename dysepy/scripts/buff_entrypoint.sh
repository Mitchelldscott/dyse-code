#! /bin/bash

#!/bin/bash
set -e

dysebash="/home/dyse-robotics/dyse-code/dysepy/dyse.bash"
echo "sourcing   	$dysebash"
source "$dysebash"

echo "Host 		$HOSTNAME"
echo "Project Root 	$PROJECT_ROOT"

exec "$@"