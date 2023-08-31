// Easing
// TODO: extract to separate file
// https://chat.openai.com/c/9e981dd3-dcc5-4d77-905c-66eaba9264c2
type EasingFunction = fn(f64) -> f64;

#[derive(Debug, Clone)]
pub struct Easing {
    start_value: f64,
    end_value: f64,

    duration_per_unit_value: f64, /* duration per unit value */
    time_step: f64,

    time: f64,

    easing_fn: EasingFunction,
}

impl Easing {
    pub fn new(
        start_value: f64,
        end_value: f64,
        duration_per_unit_value: f64,
        time_step: f64,
    ) -> Self {
        // Easing function: Ease In Ease Out Cubic (smooth start, rapid middle, smooth end)
        fn _ease_in_out_cubic(t: f64) -> f64 {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                let c = 2.0 * t - 2.0;
                0.5 * c * c * c + 1.0
            }
        }

        // Easing function: Ease Out Cubic (rapid start, smooth end)
        fn ease_out_cubic(t: f64) -> f64 {
            let t_minus_one = t - 1.0;
            t_minus_one * t_minus_one * t_minus_one + 1.0
        }

        // TODO: add bottom threshold for easing function, skip lower values ~ 0.0..0.3

        Easing {
            start_value,
            end_value,
            duration_per_unit_value,
            time_step,
            time: 0.0,
            easing_fn: ease_out_cubic,
        }
    }

    pub fn with(&self, start_value: f64, end_value: f64) -> Self {
        let mut new_value = self.clone();
        new_value.start_value = start_value;
        new_value.end_value = end_value;
        new_value.time = 0.0; // reset time
        new_value
    }

    pub fn apply_easing(&mut self) -> f64 {
        let distance = (self.end_value - self.start_value).abs(); // Calculate the distance
        let duration = distance * self.duration_per_unit_value; // Adjust the maximum duration based on distance
        self.time += self.time_step / duration; // Adjust t based on distance

        let normalized_t = self.time.min(1.0).max(0.0); // Ensure t is in the range [0, 1]
        let eased_t = (self.easing_fn)(normalized_t);
        self.start_value + (self.end_value - self.start_value) * eased_t
    }

    pub fn is_finished(&self) -> bool {
        self.time >= 1.0 || self.start_value == self.end_value
    }
}
