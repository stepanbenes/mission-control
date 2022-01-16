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

long lastLoopTime = 0;              // unit: microseconds
unsigned long loopStartTime = 0;    // unit: microseconds
float deltaTime = 0;                // unit: seconds

Mpu6050 imu = Mpu6050(MPU6050_ADDRESS);
ComplementaryFilter angleFilter; 

float currentLeanAngle = 0.0f;
float voltage = 0;              // battery voltage
bool lowVoltageTriggered = false; 
float motorSpeed = 0;           // the actual forward/reverse speed of the motors. 

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
  
  pinMode(VOLTAGE_SENSE_PIN, INPUT);
  analogReference(INTERNAL2V56);    // internal 2.54V analog voltage reference of the Arduino MEGA

  pinMode(BUZZER_PIN, OUTPUT);
  pinMode(LEFT_STEP_PIN, OUTPUT);
  pinMode(LEFT_DIR_PIN, OUTPUT);
  pinMode(LEFT_ENABLE_PIN, OUTPUT);
  pinMode(RIGHT_STEP_PIN, OUTPUT);
  pinMode(RIGHT_DIR_PIN, OUTPUT);
  pinMode(RIGHT_ENABLE_PIN, OUTPUT);

  digitalWrite(LEFT_ENABLE_PIN, HIGH); // disable the stepper motors
  digitalWrite(RIGHT_ENABLE_PIN, HIGH); // disable the stepper motors

  attachInterrupt(digitalPinToInterrupt(SONAR1_ECHO), sonar1received, FALLING);
  attachInterrupt(digitalPinToInterrupt(SONAR2_ECHO), sonar2received, FALLING);
  attachInterrupt(digitalPinToInterrupt(SONAR3_ECHO), sonar3received, FALLING);
  attachInterrupt(digitalPinToInterrupt(SONAR4_ECHO), sonar4received, FALLING);

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
  imu.updateIMUdata();

  // calculate the angle of the robot based in accelerometer and gyro data. 
  // This is done with a simple complimentary filter. See the "complimentaryFilter.h" -file. 
  float accAngle = atan2(-imu.getAccelX(), imu.getAccelZ()) * 57;       
  currentLeanAngle = angleFilter.calculate(accAngle, imu.getGyroY(), deltaTime);  
  if (SERIAL_DEBUG_SHOW_ANGLE) {
    Serial.println(currentLeanAngle);
  }

    setMotorSpeed(0, 2);  // set the speeds of the motors to zero
    setMotorSpeed(0, 1);    
    digitalWrite(LEFT_ENABLE_PIN, HIGH); // disable the stepper motors
    digitalWrite(RIGHT_ENABLE_PIN, HIGH); 

  checkBatteryVoltage(); 
  if (SERIAL_DEBUG_SHOW_BATTERY_VOLTAGE) {
    Serial.println(voltage); 
  }
  
  // MIN_BAT_VOLTAGE can be set to 0 in config file to disable low battery cut off. 
  if (MIN_BAT_VOLTAGE > 0.1f &&       // check if low battery warning is disabled in the config file
        voltage < MIN_BAT_VOLTAGE &&  // check if battery voltage is low
        voltage > 1.0f &&     // no voltage avalible, we are running from USB power, dont trigger the alarm
        millis() > 3000) {    // wait for valtage to stabilize after power on
    lowVoltageTriggered = true;   
  }
  
  // Calculate delta-time in seconds, used in PID and filter math. 
  lastLoopTime = micros() - loopStartTime;  
  deltaTime = (float)lastLoopTime / (float)1000000;  
  loopStartTime = micros();   
  if (SERIAL_DEBUG_SHOW_LOOPTIME_US) {
    Serial.println(lastLoopTime);
  }
}

// Read the battery voltage. gets stored in the global "voltage" -variable. 
// voltage is sample 3 times to minimize noise. 
void checkBatteryVoltage() {
  float value = 0;    
  for (int i = 0; i < 3; i++)
    value += analogRead(VOLTAGE_SENSE_PIN) * VOLTAGE_CALIB_FACTOR;
  voltage = value / 3.0f;
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
