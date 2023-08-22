pub struct PIDController {
    kp: f64, // Proportional gain
    ki: f64, // Integral gain
    kd: f64, // Derivative gain
    setpoint: f64,
    integral: f64,
    prev_error: f64,

    time_step: f64,
}

impl PIDController {
    pub fn new(kp: f64, ki: f64, kd: f64, setpoint: f64) -> Self {
        PIDController {
            kp,
            ki,
            kd,
            setpoint,
            integral: 0.0,
            prev_error: 0.0,
            time_step: 1.0,
        }
    }

    pub fn compute(&mut self, input: f64) -> f64 {
        let error = self.setpoint - input;

        self.integral += error * self.time_step;
        let derivative = (error - self.prev_error) / self.time_step;

        let output = self.kp * error + self.ki * self.integral + self.kd * derivative;

        self.prev_error = error;

        output
    }

    pub fn set_setpoint(&mut self, setpoint: f64) {
        self.setpoint = setpoint;
        self.integral = 0.0;
        self.prev_error = 0.0;
    }
}

impl std::default::Default for PIDController {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 0.0,
            integral: 0.0,
            prev_error: 0.0,
            time_step: 1.0,
        }
    }
}

// fn run() {
//     let mut pid = PIDController::new(1.0, 0.1, 0.01, 0.0);

//     // Simulate a control loop
//     for _ in 0..100 {
//         let current_value = // Get the current value from your system
//         let control_output = pid.compute(current_value);

//         // Apply the control output to your system
//     }
// }
