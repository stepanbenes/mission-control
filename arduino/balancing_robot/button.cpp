/*
  A class the represents a button used in user interfaces. 
  2016-09-17
*/

#include "Arduino.h"
#include "Button.h"

Button::Button(int pin) {
  _buttonPin = pin; 
  _wasPressed = _prevState = _state = false; 
  pinMode(_buttonPin, INPUT_PULLUP); 
}
    
void Button::poll() {
  _state = !digitalRead(_buttonPin);  

  if (_state == true && _prevState == false) {
    _wasPressed = true; 
  } else {
    _wasPressed = false; 
  }  

  if (_prevState &&_state)
    _heldDownTimer++;
  else
    _heldDownTimer = 0; 

  //digitalWrite(_buzzerPin, _wasPressed);  // key press buzz...
  _prevState = _state; 
}

bool Button::isDown() {
  return _state; 
}

bool Button::wasPressed() {
  return _wasPressed; 
}

bool Button::isHeldDown() {
  return (_heldDownTimer > 25);
}

