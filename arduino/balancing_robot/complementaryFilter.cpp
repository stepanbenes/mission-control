/*
  A complementary filter class used for sensor fusion 
  of accelerometer and gryo data in a balancing robot
  
  by Axel Brinkeby
  2016-08-19
*/


#include "Arduino.h"
#include "ComplementaryFilter.h"

ComplementaryFilter::ComplementaryFilter()
{
  resetValues(); 
}

double ComplementaryFilter::calculate(double newAngle, double newRate, double dt)
{
  _angle = 0.999 * (_angle + newRate * dt) + 0.001 * newAngle;   
  return _angle;  
}

void ComplementaryFilter::resetValues()
{
  _angle = 0;      
}


