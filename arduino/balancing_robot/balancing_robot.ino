/* 

  Arduino MEGA based balancing robot. Using stepper motors, driven by 16 bit hardware 
  timers and A4988 drivers. balancing is done by a PID cascade. One PID controller 
  regulates the motor speed based on the error between the current angle and a setpoint angle. 
  The other PID controller regulates the setpoint angle based on the error between the current 
  speed and a setpoint speed. An MPU-6050 IMU is used together with a complementary filter 
  to get the angle of the robot. The main loop is running at a variable loop speed, 
  as fast as possible. Delta-time is calculated every loop cycle. 

  The robot has four ultrasonic distance sensors (HC-SR04). The sensors are triggered 
  in sequence and read by pin change interrupts. The robot has a simple obstacle avoidance behavior. 
  It runs forward at a constant speed until something is to close to any of the sensors, 
  then it tries to turn away. Core for this can be found in the "behavior()" -function in this file. 

  All pin definitions can be found in the "pinConfig.h" -file. 

  All other settings including PID constants can be found in the "config.h" -file. 
  
  by Axel Brinkeby
  
  V1.0      2019-02-18
    First working version, with obctacle avoidance. 
*/
  
#include <Wire.h>                   // for comunicating the the IMU

#include "pinConfig.h"              // all the pin definitions
#include "config.h"                 // all other constansts tha affect the behavior of the robot

#include "mpu6050.h"                // IMU
#include "complementaryFilter.h"    // for accelerometer and gyro data fusion
#include "pid.h"                    // PID controller
#include "button.h"                 // for pushbuttons

long lastLoopTime = 0;              // unit: microseconds
unsigned long loopStartTime = 0;    // unit: microseconds
float deltaTime = 0;                // unit: seconds

Mpu6050 imu = Mpu6050(MPU6050_ADDRESS);
ComplementaryFilter angleFilter; 

Pid anglePID(ANGLE_P, ANGLE_I, ANGLE_D, ANGLE_I_LIMIT);
Pid speedPID(SPEED_P, SPEED_I, SPEED_D, SPEED_I_LIMIT); 

bool balancing = false; 
float currentLeanAngle = 0.0f;
float targetSpeed = 0;          // used to control the forward/reverse speed of the robot, unit: wheel revulutions per second
float turningSpeed = 0;         // used to control the turn speed of the robot, unit: wheel revulutions per second
float actualTargetSpeed = 0;    // forward/reverse speed after acceleration is applied
float actualTurningSpeed = 0;   // turn speed after acceleration is applied
float motorSpeed = 0;           // the actual forward/reverce speed of the motors. 

volatile int8_t directionMotor1 = 0;    // used in interrupt rutines to drive the motors
volatile int8_t directionMotor2 = 0;

// 8 single cycle instructions is 0.5 microsecond at 16Mhz
void delay_05us()
{
  __asm__ __volatile__ (
    "nop" "\n\t"    "nop" "\n\t"    "nop" "\n\t"    "nop" "\n\t"    
    "nop" "\n\t"    "nop" "\n\t"    "nop" "\n\t"    "nop");
}

// TIMER 1 compare interrupt for driving left motor
ISR(TIMER1_COMPA_vect)
{
  TCNT1 = 0;  
  if (directionMotor1 == 0)
    return;
    
  digitalWrite(LEFT_STEP_PIN, HIGH);
  delay_05us();
  digitalWrite(LEFT_STEP_PIN, LOW);
}

// TIMER 3 compare interrupt for driving right motor
ISR(TIMER3_COMPA_vect)
{
  TCNT3 = 0;  
  if (directionMotor2 == 0)
    return;
    
  digitalWrite(RIGHT_STEP_PIN, HIGH);
  delay_05us();
  digitalWrite(RIGHT_STEP_PIN, LOW);
}

void setup() {
  Serial.begin(SERIAL_BAUDRATE);         // for debug
  Serial.print("Initializing...");

  pinMode(ON_BOARD_LED, OUTPUT); 

  pinMode(LEFT_STEP_PIN, OUTPUT);
  pinMode(LEFT_DIR_PIN, OUTPUT);
  pinMode(LEFT_ENABLE_PIN, OUTPUT);
  pinMode(RIGHT_STEP_PIN, OUTPUT);
  pinMode(RIGHT_DIR_PIN, OUTPUT);
  pinMode(RIGHT_ENABLE_PIN, OUTPUT);

  digitalWrite(LEFT_ENABLE_PIN, HIGH); // disable the stepper motors
  digitalWrite(RIGHT_ENABLE_PIN, HIGH); // disable the stepper motors

  Wire.begin();                         
  Wire.setClock(400000);     // 400000 =  400kHz I2C clock.
  imu.init(); 
  imu.calibradeGyro();
  
  // TIMER1 controls motor 1 (left)
  TCCR1A = 0;                             // Timer1 CTC mode 4, OCxA,B outputs disconnected
  TCCR1B = (1 << WGM12) | (1 << CS11);    // Prescaler=8, => 2Mhz
  OCR1A = 65535;                          // longest period, motor stopped
  TCNT1 = 0;

  // TIMER3 controls motor 2 (right)
  TCCR3A = 0;                             // Timer1 CTC mode 4, OCxA,B outputs disconnected
  TCCR3B = (1 << WGM12) | (1 << CS11);    // Prescaler=8, => 2Mhz
  OCR3A = 65535;                          // longest period, motor stopped
  TCNT3 = 0;

  TIMSK1 |= (1 << OCIE1A);    // Enable TIMER1 interrupt
  TIMSK3 |= (1 << OCIE3A);    // Enable TIMER3 interrupt
  
  Serial.println("Done. \nStarting main loop...");
}

void loop() {
  
  // Hearbeat LED flashing, if you see this LED flashing, the robot is running it's main loop. 
  //digitalWrite(ON_BOARD_LED, millis() % 1000 < 50);

  Serial.println("huhu");
  
  delay(5000);
}

void behavior() { 
  turningSpeed = 0.0f;
  targetSpeed = 0.0f;

  // go around in a circle
  // targetSpeed = -0.5f;      
  // if (abs(motorSpeed) < 2.0f) 
  //   turningSpeed = 0.5f * targetSpeed * motorSpeed; 
  // else 
  //   turningSpeed = 0; 
}

// Set up the values used in the interrupt-functions to drive the motors.
// Speed is in steps per second. ID is 0 or 1 for left or right motor.
void setMotorSpeed(int16_t tspeed, int motorID)
{
  long timer_period;
  int16_t motorspeed = tspeed;

  noInterrupts();
  
  if (motorID == 1) {
    if (motorspeed > 0) {
      timer_period = 2000000 / motorspeed;  // 2Mhz timer interrupt
      directionMotor1 = -1;
      digitalWrite(LEFT_DIR_PIN, LOW);
    } else if (motorspeed < 0) {
      timer_period = 2000000 / -motorspeed;
      directionMotor1 = 1;
      digitalWrite(LEFT_DIR_PIN, HIGH);
    } else {
      timer_period = 65535;       // motor shuld not move at all
      directionMotor1 = 0;
    }

    if (timer_period > 65535)   // Check for maximun period without overflow
      timer_period = 65535;

    OCR1A = timer_period;  
    if (TCNT1 > OCR1A)    // Check  if we need to reset the timer...
      TCNT1 = 0; 
    
  } else if (motorID == 2){
    if (motorspeed > 0) {
      timer_period = 2000000 / motorspeed; 
      directionMotor2 = 1;
      digitalWrite(RIGHT_DIR_PIN, HIGH);
    } else if (motorspeed < 0) {
      timer_period = 2000000 / -motorspeed;
      directionMotor2 = -1;
      digitalWrite(RIGHT_DIR_PIN, LOW);
    } else {
      timer_period = 65535;
      directionMotor2 = 0;
    }

    if (timer_period > 65535) 
      timer_period = 65535;
    
    OCR3A = timer_period;  
    if (TCNT3 > OCR3A) 
      TCNT3 = 0;    
  }   
  interrupts();   
}
