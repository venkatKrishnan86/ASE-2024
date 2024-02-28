use std::f32::consts::PI;

use crate::ring_buffer::RingBuffer;
use crate::utils::{Processor, FilterParam};

pub struct Vibrato<T: Copy + Default + Into<f32>> {
    sample_rate_hz: f32,
    mod_freq: f32,
    width: usize,
    num_channels: usize,
    delay_line: Vec<RingBuffer<T>>
}

impl<T> Vibrato<T>
where T: Copy + Default + Into<f32>
{
    pub fn new(sample_rate_hz: f32, mod_freq: f32, width: T, num_channels: usize) -> Self {
        let delay = (width.into() * sample_rate_hz).round() as usize;
        let width = (width.into() * sample_rate_hz).round() as usize;
        let mod_freq: f32 = mod_freq / sample_rate_hz;
        let len_samples = 2+delay+width*2;
        let mut filter = Self {
            sample_rate_hz,
            mod_freq,
            width,
            num_channels,
            delay_line: Vec::new()
        };
        for channel in 0..filter.num_channels{
            filter.delay_line.push(RingBuffer::new(len_samples));
            filter.delay_line[channel].reset();
            for _ in 0..len_samples { filter.delay_line[channel].push(T::default()); }
        }
        filter
    }
}

impl Processor for Vibrato<f32> 
{
    type Item = f32;

    fn reset(&mut self) {
        for channel in 0..self.num_channels{
            self.delay_line[channel].reset();
            for _ in 0..(2 + 3*self.width) {
                self.delay_line[channel].push(Self::Item::default());
            }
        }
    }

    fn get_param(&self, param: FilterParam) -> Self::Item {
        match param {
            FilterParam::ModFreq => self.mod_freq * self.sample_rate_hz,
            FilterParam::Width => self.width as f32/self.sample_rate_hz
        }
    }

    fn process(&mut self, input: &[&[Self::Item]], output: &mut[&mut[Self::Item]]) {
        for (channel, (input_channel, output_channel)) in input.iter().zip(output.iter_mut()).enumerate() {
            for (sample_index, (input_sample, output_sample)) in input_channel.iter().zip(output_channel.iter_mut()).enumerate() {
                let modulator = (self.mod_freq * 2.0 * PI * (sample_index+1) as f32).sin();
                let tap = 1.0 + self.width as f32 + self.width as f32 * modulator;
                self.delay_line[channel].pop();
                self.delay_line[channel].push(*input_sample);
                *output_sample = self.delay_line[channel].get_frac(tap);
            }
        }
    }

    fn set_param(&mut self, param: FilterParam, value: Self::Item) -> Result<(), String> {
        if value <= 0.0 {
            return Err("Value must be positive!".to_owned())
        }
        match param {
            FilterParam::ModFreq => self.mod_freq = value / self.sample_rate_hz,
            FilterParam::Width => self.width = (value * self.sample_rate_hz).round() as usize
        }
        Ok(())
    }
}

// impl Processor for Vibrato<i16> 
// {
//     type Item = i16;

//     fn reset(&mut self) {
//         todo!("");
//     }

//     fn get_param(&self, param: FilterParam) -> Self::Item {
//         match param {
//             FilterParam::ModFreq => self.mod_freq as i16,
//             FilterParam::Width => self.width
//         }
//     }

//     fn process(&mut self, input: &[&[Self::Item]], output: &mut[&mut[Self::Item]]) {
        
//     }

//     fn set_param(&mut self, param: FilterParam, value: Self::Item) -> Result<(), String> {
//         if value <= 0 {
//             return Err("Value must be positive!".to_owned())
//         }
//         match param {
//             FilterParam::ModFreq => self.mod_freq = value as f32,
//             FilterParam::Width => self.width = value
//         }
//         Ok(())
//     }
// }