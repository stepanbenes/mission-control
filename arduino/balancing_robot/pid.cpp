/*
  A basic PID class. 
  by Axel Brinkeby.
  2016-05-08
*/

#include "Arduino.h"
#include "pid.h"

Pid::Pid(float p, float i, float d, float iLimit)
{
  _p = p; 
  _i = i; 
  _d = d; 
  _i_limit = iLimit; 
  _integratedError = 0; 
  _lastError = 0; 
}

float Pid::updatePID(float target, float current, float deltaTime)
{
  float error = (target - current) * deltaTime; 
  
  float pPart = _p * error;
  
  _integratedError += error;    
  _integratedError = constrain(_integratedError, -_i_limit, _i_limit);
  float iPart = _i * _integratedError; 
  
  float dPart = _d * (error - _lastError);    
  
  _lastError = error;
   
  return (pPart + iPart + dPart); 
}

void Pid::resetPID()
{
  _integratedError = 0; 
  _lastError = 0; 
}

void Pid::setP(float p)
{
  _p = p; 
}

void Pid::setI(float i)
{
  _i = i; 
} 

void Pid::setD(float d)
{
  _d = d; 
}

void Pid::setIlimit(float limit)
{
  _i_limit = limit; 
} 

float Pid::getP()
{
  return _p;  
}

float Pid::getI()
{
  return _i;  
} 

float Pid::getD()
{
  return _d;  
}
 
float Pid::getIlimit()
{
  return _i_limit;  
}     


