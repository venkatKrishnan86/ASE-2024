use std::f32::consts::PI;

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
    use super::*;

    #[test]
    fn test_1() {
        let lfo = LFO::new(5.0, 10.0);
    }
}