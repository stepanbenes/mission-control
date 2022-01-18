/*
A simple class to control an RGB LED using three pins. 
*/

#ifndef RGBled_h
#define RGBled_h

#include "Arduino.h"

class RGBled
{
  public: 
    RGBled(int Rpin, int Gpin, int Bpin); 

    void setColor(float R, float G, float B);
     
    void update();
      
  private: 

    int _Rpin, _Gpin, _Bpin;
    int _Rvalue, _Gvalue, _Bvalue;
    int _Rpwm, _Gpwm, _Bpwm;
};

#endif
