/*
A simple class to control an RGB LED using three pins. 
*/

#include "Arduino.h"
#include "rgbled.h"

RGBled::RGBled(int Rpin, int Gpin, int Bpin) {
  _Rpin = Rpin; 
  _Gpin = Gpin; 
  _Bpin = Bpin;
  pinMode(_Rpin, OUTPUT);
  pinMode(_Gpin, OUTPUT);
  pinMode(_Bpin, OUTPUT);
}

void RGBled::setColor(float R, float G, float B) {
  _Rvalue = R; 
  _Gvalue = G; 
  _Bvalue = B; 
  _Rpwm = constrain(255 - R * 255, 0, 255); 
  _Gpwm = constrain(255 - G * 255, 0, 255); 
  _Bpwm = constrain(255 - B * 255, 0, 255);    
}
    
void RGBled::update() {
  analogWrite(_Rpin, _Rpwm);
  analogWrite(_Gpin, _Gpwm);
  analogWrite(_Bpin, _Bpwm);
}
