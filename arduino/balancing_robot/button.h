/*
  A class the represents a button used in user interfaces. 
  2016-09-17
*/

#ifndef Button_h
#define Button_h

#include "Arduino.h"

class Button
{
  public: 
    Button(int pin); 
    
    void poll(); 
    bool isDown(); 
    bool wasPressed(); 
    bool isHeldDown();
      
  private: 

    int _buttonPin; 
    bool _state, _prevState, _wasPressed; 
    int _heldDownTimer;
};

#endif
