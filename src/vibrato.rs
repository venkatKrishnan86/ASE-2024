use crate::ring_buffer::RingBuffer;
use crate::utils::{Processor, FilterParam};
use crate::lfo;

pub struct Vibrato
{
    sample_rate_hz: f32,
    width: usize,
    num_channels: usize,
    lfo: Vec<lfo::LFO>,
    delay_line: Vec<RingBuffer<f32>>
}

impl Vibrato
{
    pub fn new(sample_rate_hz: f32, mod_freq: f32, width: f32, num_channels: usize) -> Self {
        let width = (width * sample_rate_hz).round() as usize;
        let len_samples = 2+width*3;
        let mut filter = Self {
            sample_rate_hz,
            width,
            num_channels,
            lfo: vec![lfo::LFO::new(mod_freq, sample_rate_hz); num_channels],
            delay_line: vec![RingBuffer::new(len_samples); num_channels]
        };
        for channel in filter.delay_line.iter_mut(){
            channel.set_read_index(0);
            channel.set_write_index(len_samples-1);
        }
        filter
    }
}

impl Processor for Vibrato
{
    type Item = f32;

    fn reset(&mut self) {
        for channel in 0..self.num_channels{
            self.delay_line[channel].reset();
            self.lfo[channel].reset();
            for _ in 0..(2 + 3*self.width) {
                self.delay_line[channel].push(Self::Item::default());
            }
        }
    }

    fn get_param(&self, param: FilterParam) -> Self::Item {
        match param {
            FilterParam::ModFreq => {
                (1.0/self.lfo[0].size() as f32) * self.sample_rate_hz
            },
            FilterParam::Width => self.width as f32/self.sample_rate_hz
        }
    }

    fn process(&mut self, input: &[&[Self::Item]], output: &mut[&mut[Self::Item]]) {
        for (channel, (input_channel, output_channel)) in input.iter().zip(output.iter_mut()).enumerate() {
            // ISSUE: len_samples DO NOT match the N-2 criteria in the loop
            for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter_mut()) {
                let modulator = self.lfo[channel].output_sample();
                let offset = 1.0 + self.width as f32 + self.width as f32 * modulator;
                let _ = self.delay_line[channel].pop();
                self.delay_line[channel].push(*input_sample);
                *output_sample = self.delay_line[channel].get_frac(offset);
            }
        }
    }

    fn set_param(&mut self, param: FilterParam, value: Self::Item) -> Result<(), String> {
        if value <= 0.0 {
            return Err("Value must be positive!".to_owned())
        }
        match param {
            FilterParam::ModFreq => {
                for channel in 0..self.num_channels {
                    self.lfo[channel].modify_mod_freq(value);
                }
            }
            FilterParam::Width => {
                self.width = (value * self.sample_rate_hz).round() as usize;
                self.delay_line = vec![RingBuffer::new(2 + self.width*3); self.num_channels];
            }
        }
        self.reset();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::is_close; 

    mod set_param_tests {
        use std::fmt::format;

        use super::*;

        /// This test is for checking if the setting `ModFreq` parameter changes the value to the requested value
        ///
        /// ## Returns
        ///
        /// Result<(), String> : This is to ensure the `test_2_mod_freq()` runs only if this test passes
        #[test]
        fn test_1_mod_freq() -> Result<(), String>{
            let mut vib = Vibrato::new(44100.0, 25.0, 0.002, 2);
            let _ = vib.set_param(FilterParam::ModFreq, 12.0);
            if !(is_close(12.0, vib.get_param(FilterParam::ModFreq), 0.001)) {
                Err(String::from("Mod Frequency set_param for Vibrato is wrong!"))
            }
            else {
                Ok(())
            }
        }

        /// This test is for checking whether the `sine_buffer` in `LFO` is *recreated* after modifying the `ModFreq` parameter 
        ///
        /// NOTE: Will only run if `test_1_mod_freq()` passes
        #[test]
        fn test_2_mod_freq() {
            if let Ok(()) = test_1_mod_freq() {
                let mut vib = Vibrato::new(44100.0, 25.0, 0.002, 2);
                let _ = vib.set_param(FilterParam::ModFreq, 12.0);
                let value = f32::round(44100.0/12.0) as usize;
                assert_eq!(value, vib.lfo[0].size());
            }
            else {
                panic!("test_1_mod_freq() must pass first!")
            }
        }

        /// This test is for checking if the setting `Width` parameter changes the value to the requested value
        ///
        /// ## Returns
        ///
        /// Result<(), String> : This is to ensure the `test_2_width()` runs only if this test passes
        #[test]
        fn test_1_width() -> Result<(), String> {
            let mut vib = Vibrato::new(44100.0, 25.0, 0.002, 2);
            let _ = vib.set_param(FilterParam::Width, 12.0);
            if !(is_close(12.0, vib.get_param(FilterParam::Width), 0.001)){
                return Err(String::from("Mod Frequency set_param for Vibrato is wrong!"))
            }
            Ok(())
        }

        /// This test is for checking whether the `delay_line` in `Vibrato` is *recreated* after modifying the `Width` parameter 
        ///
        /// NOTE: Will only run if `test_1_width()` passes
        #[test]
        fn test_2_width() {
            if let Ok(()) = test_1_width() {
                let mut vib = Vibrato::new(44100.0, 25.0, 0.002, 2);
                let _ = vib.set_param(FilterParam::Width, 12.0);
                for channel in 0..vib.num_channels {
                    assert_eq!(2 + vib.width*3, vib.delay_line[channel].len());
                }
            }
            else {
                panic!("test_1_width() must pass first!")
            }
        }
    }

    mod get_param_tests {
        use super::*;

        #[test]
        fn test_1_mod_freq() {
            let vib = Vibrato::new(44100.0, 25.0, 0.002, 2);
            assert!(is_close(25.0, vib.get_param(FilterParam::ModFreq), 0.001), "Mod Frequency get_param for Vibrato is wrong!");
        }

        #[test]
        fn test_1_width() {
            let vib = Vibrato::new(44100.0, 25.0, 0.002, 2);
            assert!(is_close(0.002, vib.get_param(FilterParam::Width), 0.001), "Width get_param for Vibrato is wrong!");
        }
    }

    mod process_tests {
        use super::*;

        #[test]
        fn test_1() {
            todo!("Take from comb filter")
        }
    }
}