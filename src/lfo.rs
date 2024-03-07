use std::f32::consts::PI;

// Premake the LFO sine, and use mod frequency as a phase value to get_frac values

#[derive(Clone)]
pub struct LFO {
    sine_buffer: Vec<f32>,
    sample_rate: f32,
    curr_index: usize,
    capacity: usize
}

impl LFO {
    pub fn new(mod_freq: f32, sample_rate: f32) -> Self {
        let mod_freq = mod_freq/sample_rate;
        let capacity = (1.0/mod_freq).round() as usize;
        let mut sine_buffer = Vec::new();
        for value in 1..=capacity {
            sine_buffer.push((mod_freq * 2.0 * PI * value as f32).sin());
        }
        Self {
            sine_buffer: sine_buffer,
            sample_rate: sample_rate,
            curr_index: 0,
            capacity: capacity
        }
    }

    pub fn modify_mod_freq(&mut self, new_mod_freq: f32) {
        self.capacity = (self.sample_rate/new_mod_freq).round() as usize;
        self.sine_buffer = Vec::new();
        for value in 1..=self.capacity {
            self.sine_buffer.push((new_mod_freq * 2.0 * PI * value as f32).sin());
        }
    }

    pub fn reset(&mut self) {
        self.curr_index = 0;
    }

    pub fn output_sample(&mut self) -> f32 {
        let temp = self.sine_buffer[self.curr_index];
        self.curr_index = (self.curr_index+1)%self.capacity;
        return temp;
    }

    pub fn size(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::is_close;
    use super::*;

    /// This test is for the new() function
    /// 
    /// Asserts that the sine_buffer in the LFO is created properly
    #[test]
    fn test_1() -> Result<(), String> {
        let lfo = LFO::new(5.0, 20.0);
        let sine: Vec<f32> = vec![1.0, 0.0, -1.0, 0.0];
        for (true_val, lfo_val) in sine.into_iter().zip(lfo.sine_buffer.into_iter()){
            if !(is_close(true_val, lfo_val, 1e-5)){
                return Err(format!("{true_val} doees not match the LFO value {lfo_val}").to_owned());
            };
        }
        Ok(())
    }

    /// This test is for the output_sample() function
    /// 
    /// Asserts that the output_sample() must repeatedly return the values in the sine_buffer in a cyclic format without an end value
    #[test]
    fn test_2() {
        if let Ok(()) = test_1() {
            let mut lfo = LFO::new(5.0, 20.0);
            let sine: Vec<f32> = vec![1.0, 0.0, -1.0, 0.0];
            for index in 0..16 {
                assert!(is_close(lfo.output_sample(), sine[index%4 as usize], 1e-5), "Sample does not match at {index}");
            }
        }
        else {
            panic!("test_1() must pass first!")
        }
    }
}