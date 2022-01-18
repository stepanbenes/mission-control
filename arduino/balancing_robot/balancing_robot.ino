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
#include "rgbled.h"                 // controls the RGB LED "eyes", used as status indicators

long lastLoopTime = 0;              // unit: microseconds
unsigned long loopStartTime = 0;    // unit: microseconds
float deltaTime = 0;                // unit: seconds

Mpu6050 imu = Mpu6050(MPU6050_ADDRESS);
ComplementaryFilter angleFilter; 

Pid anglePID(ANGLE_P, ANGLE_I, ANGLE_D, ANGLE_I_LIMIT);
Pid speedPID(SPEED_P, SPEED_I, SPEED_D, SPEED_I_LIMIT); 

RGBled leftRGB(LEFT_RGB_RED, LEFT_RGB_GREEN, LEFT_RGB_BLUE); 
RGBled rightRGB(RIGHT_RGB_RED, RIGHT_RGB_GREEN, RIGHT_RGB_BLUE);
RGBled balanceStatus(STATUS_LED_FRONT, STATUS_LED_CENTER, STATUS_LED_REAR);

bool balancing = false; 
float currentLeanAngle = 0.0f;
float targetSpeed = 0;          // used to control the forward/reverse speed of the robot, unit: wheel revulutions per second
float turningSpeed = 0;         // used to control the turn speed of the robot, unit: wheel revulutions per second
float actualTargetSpeed = 0;    // forward/reverse speed after acceleration is applied
float actualTurningSpeed = 0;   // turn speed after acceleration is applied
float voltage = 0;              // battery voltage
bool lowVoltageTriggered = false; 
float motorSpeed = 0;           // the actual forward/reverce speed of the motors. 

volatile int8_t directionMotor1 = 0;    // used in interrupt rutines to drive the motors
volatile int8_t directionMotor2 = 0;

volatile long sonarStart[] = {0, 0, 0, 0};    // times in microseconds the last time the sensors 
volatile float sonarValue[] = {0, 0, 0, 0};     // normalized values from the sensors, 1 = obstacle, 0 = no obstacle

long LastSonarFire = 0;       // milliseconds time of last sonar fire
int sonarIndexCounter = 0;    // keeps track of which sensor to fire next

// Those variables are used for obstacle ovaidance behavior with ultrasonic sensors
int turnCounter = 0; 
int forwardCounter = 0; 
float turnMultiplyer = 1; 

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

void sonar1received() {
  long sonarTime = micros() - sonarStart[0]; 
  sonarValue[0] = normalizedSonarDistance(sonarDistance(sonarTime));
}

void sonar2received() {
  long sonarTime = micros() - sonarStart[1]; 
  sonarValue[1] = normalizedSonarDistance(sonarDistance(sonarTime));
}

void sonar3received() {
  long sonarTime = micros() - sonarStart[2]; 
  sonarValue[2] = normalizedSonarDistance(sonarDistance(sonarTime));
}

void sonar4received() {
  long sonarTime = micros() - sonarStart[3]; 
  sonarValue[3] = normalizedSonarDistance(sonarDistance(sonarTime));
}

// Returns the sonar distance in centimeters.
float sonarDistance(long echoTime) {
  return echoTime / 58.2f;
}

// Retruns a value between 0 and 1, where 0 is "no obstacle detected" and 1 is "obstacle is very close by"
float normalizedSonarDistance(float distance) {
  float maxDist = 80; 
  return constrain(((maxDist - distance) / maxDist), 0.0f, 1.0f);
}

void setup() {
  Serial.begin(SERIAL_BAUDRATE);         // for debug
  Serial.print("Initializing...");
  
  pinMode(VOLTAGE_SENSE_PIN, INPUT);
  analogReference(INTERNAL2V56);    // internal 2.54V analog voltage reference of the Arduino MEGA
  
  pinMode(STATUS_LED_FRONT, OUTPUT);
  pinMode(STATUS_LED_CENTER, OUTPUT);
  pinMode(STATUS_LED_REAR, OUTPUT);
  
  pinMode(ON_BOARD_LED, OUTPUT); 

  leftRGB.setColor(1, 0, 0);   leftRGB.update();
  rightRGB.setColor(1, 0, 0);  rightRGB.update();

  pinMode(BUZZER_PIN, OUTPUT);
  pinMode(LEFT_STEP_PIN, OUTPUT);
  pinMode(LEFT_DIR_PIN, OUTPUT);
  pinMode(LEFT_ENABLE_PIN, OUTPUT);
  pinMode(RIGHT_STEP_PIN, OUTPUT);
  pinMode(RIGHT_DIR_PIN, OUTPUT);
  pinMode(RIGHT_ENABLE_PIN, OUTPUT);

  digitalWrite(LEFT_ENABLE_PIN, HIGH); // disable the stepper motors
  digitalWrite(RIGHT_ENABLE_PIN, HIGH); // disable the stepper motors

  pinMode(SONAR1_TRIG, OUTPUT);
  pinMode(SONAR2_TRIG, OUTPUT);
  pinMode(SONAR3_TRIG, OUTPUT);
  pinMode(SONAR4_TRIG, OUTPUT);
  
  pinMode(SONAR1_ECHO, INPUT);
  pinMode(SONAR2_ECHO, INPUT);
  pinMode(SONAR3_ECHO, INPUT);
  pinMode(SONAR4_ECHO, INPUT);

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
  
  tone(BUZZER_PIN, 200, 200); delay(200); 
  tone(BUZZER_PIN, 500, 400); delay(500); 
  Serial.println("Done. \nStarting main loop...");
}

void loop() {
  
  // Hearbeat LED flashing, if you see this LED flashing, the robot is running it's main loop. 
  digitalWrite(ON_BOARD_LED, millis() % 500 < 50);

  if (LastSonarFire + 20 < millis()) {
    LastSonarFire = millis();
    
    if(sonarIndexCounter == 0) {     
      digitalWrite(SONAR1_TRIG, HIGH); 
      delayMicroseconds(2);   
      digitalWrite(SONAR1_TRIG, LOW);
    } else if(sonarIndexCounter == 1) {     
      digitalWrite(SONAR2_TRIG, HIGH); 
      delayMicroseconds(2);   
      digitalWrite(SONAR2_TRIG, LOW);
    } else if(sonarIndexCounter == 2) {     
      digitalWrite(SONAR3_TRIG, HIGH); 
      delayMicroseconds(2);   
      digitalWrite(SONAR3_TRIG, LOW);
    } else if(sonarIndexCounter == 3) {     
      digitalWrite(SONAR4_TRIG, HIGH); 
      delayMicroseconds(2);   
      digitalWrite(SONAR4_TRIG, LOW);
    }     
    sonarStart[sonarIndexCounter] = micros(); 

    sonarIndexCounter++;
    if (sonarIndexCounter >= 4) {
      sonarIndexCounter = 0; 
    }
  }
  
  if (SERIAL_DEBUG_SHOW_SONAR) {
    for (int i = 0; i < 4; i++) {
      Serial.print(sonarValue[i]);
      Serial.print("\t");
    }
    Serial.print("\n");
  }

  imu.updateIMUdata();

  // calculate the angle of the robot based in accelerometer and gyro data. 
  // This is done with a simple complimentary filter. See the "complimentaryFilter.h" -file. 
  float accAngle = atan2(-imu.getAccelX(), imu.getAccelZ()) * 57;       
  currentLeanAngle = angleFilter.calculate(accAngle, imu.getGyroY(), deltaTime);  
  if (SERIAL_DEBUG_SHOW_ANGLE)  
    Serial.println(currentLeanAngle);

  behavior(); // controls how the robot chuld move. This function contains the obstacle avoidance behavior

  actualTargetSpeed += constrain(targetSpeed - actualTargetSpeed, -SPEED_ACCELERATION, SPEED_ACCELERATION); 
  actualTurningSpeed += constrain(turningSpeed -actualTurningSpeed, -TURN_ACCELERATION, TURN_ACCELERATION);  

  // balance the robot.   
  float targetAngle = speedPID.updatePID(actualTargetSpeed, motorSpeed, deltaTime);
  motorSpeed = -anglePID.updatePID(targetAngle, currentLeanAngle, deltaTime);

  // start balancing if the robot is close to equilibrium. 
  if (!lowVoltageTriggered && !balancing && millis() > 3000 && abs(currentLeanAngle) < START_ANGLE_ERROR) {
    balancing = true;  
    turningSpeed = motorSpeed = forwardCounter = turnCounter = 0; 
    anglePID.resetPID(); 
    speedPID.resetPID(); 
    angleFilter.resetValues();
    tone(BUZZER_PIN, 500, 100);
  }
  
  if (balancing) {
    // Stop balancing if angle error is to large. 
    if ((targetAngle - MAX_ACCEPTABLE_ANGLE_ERROR) > currentLeanAngle || 
          currentLeanAngle > (targetAngle + MAX_ACCEPTABLE_ANGLE_ERROR)) {
      balancing = false;         
      tone(BUZZER_PIN, 200, 100);
    }

    // Stop balancing if the robot is leaning to much in any direction.
    if (-MAX_ACCEPTABLE_ANGLE > currentLeanAngle || currentLeanAngle > MAX_ACCEPTABLE_ANGLE) {
      balancing = false;     
      tone(BUZZER_PIN, 200, 100);
    }
    
    // Motor speed is converted from rotations per second to steps per second
    int16_t leftMotorSpeed = (motorSpeed + actualTurningSpeed) * 3600;
    int16_t rightMotorSpeed = (motorSpeed - actualTurningSpeed) * 3600; 
    setMotorSpeed(leftMotorSpeed, 2);
    setMotorSpeed(rightMotorSpeed, 1);
    digitalWrite(LEFT_ENABLE_PIN, LOW); // enable the stepper motors
    digitalWrite(RIGHT_ENABLE_PIN, LOW);
  } 
  else      // robot is not balancing
  {
    setMotorSpeed(0, 2);  // set the speeds of the motors to zero
    setMotorSpeed(0, 1);    
    digitalWrite(LEFT_ENABLE_PIN, HIGH); // disable the stepper motors
    digitalWrite(RIGHT_ENABLE_PIN, HIGH); 
    leftRGB.setColor(0, 1, 0);
    rightRGB.setColor(0, 1, 0);
  }

  digitalWrite(STATUS_LED_FRONT, currentLeanAngle > START_ANGLE_ERROR); 
  digitalWrite(STATUS_LED_CENTER, abs(currentLeanAngle) < START_ANGLE_ERROR); 
  digitalWrite(STATUS_LED_REAR, currentLeanAngle < -START_ANGLE_ERROR); 
   
  leftRGB.update();           // left RGB LED "eye"
  rightRGB.update();          // right RGB LED "eye"
  
  checkBatteryVoltage(); 
  if (SERIAL_DEBUG_SHOW_BATTERY_VOLTAGE) {
    Serial.println(voltage); 
  }
  
  // MIN_BAT_VOLTAGE can be set to 0 in config file to disable low battery cut off. 
  if (MIN_BAT_VOLTAGE > 0.1f &&       // check if low battery warning is disabled in the config file
        voltage < MIN_BAT_VOLTAGE &&  // check if battery voltage is low
        voltage > 1.0f &&     // no voltage avalible, we are running from USB power, dont trigger the alarm
        millis() > 3000) {    // wait for valtage to stabilize after power on
    balancing = false;  
    lowVoltageTriggered = true;   
  }

  if (lowVoltageTriggered && (millis() % 1500 < 50)) {    // Low voltage alarm buzzer sound. 
    tone(BUZZER_PIN, 200, 200);    
  }
  
  // Calculate delta-time in seconds, used in PID and filter math. 
  lastLoopTime = micros() - loopStartTime;  
  deltaTime = (float)lastLoopTime / (float)1000000;  
  loopStartTime = micros();   
  if (SERIAL_DEBUG_SHOW_LOOPTIME_US) 
    Serial.println(lastLoopTime); 
}

void behavior() { 
  if (forwardCounter < 0) {       // go reverse
    forwardCounter++; 
    targetSpeed = 0.4f;
    leftRGB.setColor(1, 0, 0);
    rightRGB.setColor(1, 0, 0);
    
  } else if (turnCounter > 0) {            // turning to the right
    turningSpeed = 0.6f;
    targetSpeed = 0.0f;
    turnCounter--; 
    leftRGB.setColor(0, 1, 1);
    rightRGB.setColor(1, 1, 0);
    
  } else if (turnCounter < 0) {       // turning to the left
    turningSpeed = -0.6f;
    targetSpeed = 0.0f;
    turnCounter++; 
    leftRGB.setColor(1, 1, 0);
    rightRGB.setColor(0, 1, 1);
    
  } else {                        // going straight forward
    forwardCounter++; 
    if (forwardCounter > 500) {
      turnMultiplyer = 1; 
    }

    leftRGB.setColor(0, 1, 1);
    rightRGB.setColor(0, 1, 1);
    
    turningSpeed = 0;
    targetSpeed = -0.8f * constrain((float)forwardCounter / 1500.0f, 0.05f, 1.0f);

    if (sonarValue[0] > 0.05f || sonarValue[1] > 0.05f) {    // turn to the right while going forward
      turningSpeed = 0.3f;  
      //targetSpeed = -0.5f;  
           
      leftRGB.setColor(0, 1, 1);   
      rightRGB.setColor(1, 0, 0);
    
    } else if (sonarValue[2] > 0.05f || sonarValue[3] > 0.05f) {   // turn to the left while going forward 
      turningSpeed = -0.3f; 
      //targetSpeed = -0.5f;  
      
      leftRGB.setColor(1, 0, 0);
      rightRGB.setColor(0, 1, 1);
    }

    
    if (sonarValue[0] > 0.7f || sonarValue[1] > 0.7f || sonarValue[2] > 0.7f || sonarValue[3] > 0.7f) {
      forwardCounter = -500;

      
    } else if (sonarValue[0] > 0.3f) {             // shuld start a turn to the left
      turnCounter = 200 * turnMultiplyer;
      turnMultiplyer -= 0.1f; 
      forwardCounter = 0; 
      
    } else if (sonarValue[1] > 0.3f) {      // shuld start a turn to the left
      turnCounter = 100 * turnMultiplyer;  
      turnMultiplyer -= 0.1f;
      forwardCounter = 0; 
      
    } else if (sonarValue[2] > 0.3f) {      // shuld start a turn to the right  
      turnCounter = -100 * turnMultiplyer;
      turnMultiplyer -= 0.1f;
      forwardCounter = 0; 
      
    } else if (sonarValue[3] > 0.3f) {      // shuld start a turn to the right  
      turnCounter = -200 * turnMultiplyer;   
      turnMultiplyer -= 0.1f; 
      forwardCounter = 0;    
    }

    if (turnMultiplyer < 0.15f) {
      turnCounter = 400; 
      turnMultiplyer = 1; 
    }
  }

  /*
  // go around in a circle
  targetSpeed = -0.5f;      
  if (abs(motorSpeed) < 2.0f) 
    turningSpeed = 0.5f * targetSpeed * motorSpeed; 
  else 
    turningSpeed = 0; 
  */
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
