// I2C address of the IMU
const int MPU6050_ADDRESS = 0x68;

// PID constants for the angle PID controller
const float ANGLE_P = 30.0;            
const float ANGLE_I = 2.5;         
const float ANGLE_D = 40.0;           
const float ANGLE_I_LIMIT = 15;      

// PID constants for the speed PID controller
const float SPEED_P = 800;     
const float SPEED_I = 20.0;      
const float SPEED_D = 0.0;      
const float SPEED_I_LIMIT = 20;      

// The robot will stop balancing if the angle error between the 
// target angle and the current angle gets larger than this value
const float MAX_ACCEPTABLE_ANGLE_ERROR = 50; 

// The robot will stop balancing it at leans more than this angle. 
const float MAX_ACCEPTABLE_ANGLE = 20; 

// if the angle error is less than this value, the robot will try to start balancing.
const float START_ANGLE_ERROR = 3; 

// it the battery voltage goes below this value, the robot will stop balancing. 
const float MIN_BAT_VOLTAGE = 10.5f;

// battery voltage calibration value. The result from the analogRead -function
// is multiplied by this value to get the battery voltage. 
const float VOLTAGE_CALIB_FACTOR = 0.02499f;

// Robot acceleration, this is not motor acceleration. This contols how fast 
// the robot can change its turning and forward/reverse traget speed. 
const float SPEED_ACCELERATION = 0.003; 
const float TURN_ACCELERATION = 0.003; 

// Serial consol debug flags. Enable to show values in the serial console. 
// to make it easy to read, only enable on at a time. 
const long SERIAL_BAUDRATE = 9600; 

// The angle of the robot, close to 0 is stright up, 
// negative values leaning forward, positive values leaning backwards. 
const bool SERIAL_DEBUG_SHOW_ANGLE = true; 

// Show the normalized sonar distances of the four sonar sensors
// 0 is no abstacle detected, 1 means obstable is very close
const bool SERIAL_DEBUG_SHOW_SONAR = false; 

// Voltage of the main battery, use this to check that it is correctly calibrated. 
// adjust the "VOLTAGE_CALIB_FACTOR" until this reads the same voltage as a connected multimeter
const bool SERIAL_DEBUG_SHOW_BATTERY_VOLTAGE = false;

// The time in microseconds to make one cycly of the main loop
const bool SERIAL_DEBUG_SHOW_LOOPTIME_US = false;
