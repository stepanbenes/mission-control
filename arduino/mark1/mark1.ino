// https://github.com/waspinator/AccelStepper/blob/master/src/AccelStepper.h
// https://dronebotworkshop.com/stepper-motors-with-arduino/
// https://www.youtube.com/watch?v=0qwrnUeSpYQ

#include "limits.h"
#include <AccelStepper.h>

// Define step constants
#define FULLSTEP 4
#define HALFSTEP 8
 
// Define Motor Pins (2 Motors used)
 
#define motorPin1  8 
#define motorPin2  9 
#define motorPin3  10
#define motorPin4  11

#define motor2Pin1  7 
#define motor2Pin2  6 
#define motor2Pin3  5
#define motor2Pin4  4

#define trigPin 12    // Trigger
#define echoPin 13    // Echo
           
// Define two motor objects
// The sequence 1-3-2-4 is required for proper sequencing of 28BYJ48
AccelStepper stepper1(FULLSTEP, motorPin1, motorPin3, motorPin2, motorPin4);
AccelStepper stepper2(FULLSTEP, motor2Pin1, motor2Pin3, motor2Pin2, motor2Pin4);

#define MAX_SPEED 600 // 620?

enum Mode {
  Off,
  IndefiniteConstantSpeed,
  AccelerateTowardsDestination
};

Mode mode = Mode::Off;

void setup() {
  Serial.begin(9600);
  Serial.println("Hello.");

  // 1 revolution Motor 1 CW
  stepper1.setMaxSpeed(1000.0);
  stepper1.setAcceleration(200.0);
  //stepper1.setSpeed(50);

  stepper2.setMaxSpeed(1000.0);
  stepper2.setAcceleration(200.0);

  pinMode(trigPin, OUTPUT);
  pinMode(echoPin, INPUT);
}

void loop() {
  while (Serial.available() > 0)
  {
    char c = Serial.read();
    switch (c)
    {
      case 'f':
        {
          mode = Mode::IndefiniteConstantSpeed;

          // TODO: set speed, read value from serial
          stepper1.setSpeed(MAX_SPEED);
          stepper2.setSpeed(MAX_SPEED);
        }
        break;
      case 'a':
        {
          mode = Mode::AccelerateTowardsDestination;
          stepper1.setMaxSpeed(MAX_SPEED);
          stepper1.setAcceleration(50.0);
          stepper1.setSpeed(0);
          stepper1.setCurrentPosition(0);
          
          // TODO: set desired position, read value from serial
          stepper1.moveTo(2048); // one turn has 2048 full steps.
    
          stepper2.setMaxSpeed(MAX_SPEED);
          stepper2.setAcceleration(50.0);
          stepper2.setSpeed(0);
          stepper2.setCurrentPosition(0);
          stepper2.moveTo(2048); // one turn has 2048 full steps.
        }
        break;
      case 's':
        {
          mode = Mode::Off;
          stepper1.stop();
          stepper2.stop();
        }
        break;
      case 'd':
        {
          // The sensor is triggered by a HIGH pulse of 10 or more microseconds.
          // Give a short LOW pulse beforehand to ensure a clean HIGH pulse:
          digitalWrite(trigPin, LOW);
          delayMicroseconds(5);
          digitalWrite(trigPin, HIGH);
          delayMicroseconds(10);
          digitalWrite(trigPin, LOW);
          
          // Read the signal from the sensor: a HIGH pulse whose
          // duration is the time (in microseconds) from the sending
          // of the ping to the reception of its echo off of an object.
          pinMode(echoPin, INPUT);
          long duration = pulseIn(echoPin, HIGH);
          // Convert the time into a distance
          long mm = ((double)duration / 2) * 0.343;
          Serial.println(mm);
        }
        break;
    }
  }
  
  switch (mode) {
    case Mode::IndefiniteConstantSpeed:
      stepper1.runSpeed();
      stepper2.runSpeed();
      break;
    case Mode::AccelerateTowardsDestination:
      if (stepper1.distanceToGo() == 0 && stepper2.distanceToGo() == 0) {
        mode = Mode::Off;
        Serial.println("stop");
      }
      else {
        stepper1.run();
        stepper2.run();
      }
      break;
  }
}
