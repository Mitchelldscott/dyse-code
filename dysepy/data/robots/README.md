# Robot Descriptions
Yaml files containing the nodes and data that make up a robot. Almost a rip off of roslaunch

## Adding a robot

1. Modify `robots.yaml` to have the IP and type of your robot
    
    IP_ADDRESS:
      'ROBOT_TYPE'

## Setting a machine's default robot

1. Change `self.txt` to have the name of the robot (must be a folder in this directory)

## Adding a robot definition

1. Copy penguin/ into a directory with the name of your robot
2. Swap the urdf for your own
3. Setup firmware_tasks.yaml
    - Sensors first
    - Then test the actuators by setting output through hid
    - build, add and test internal firmware tasks
    - connect internal tasks to sensors and validate
    - connect test output to actuator tasks and validate
    - connect internal tasks to actuators
4. Implement and add nodes to nodes.yaml
5. in a bash terminal: 

    run <robot_name>