use crate::ring_buffer::RingBuffer;
use crate::utils::{Processor, FilterParam};
use crate::lfo::lfo;

pub struct Vibrato
{
    sample_rate_hz: f32,
    mod_freq: f32,
    width: usize,
    num_channels: usize,
    delay_line: Vec<RingBuffer<f32>>,
    sample_index: Vec<usize>
}

impl Vibrato
{
    pub fn new(sample_rate_hz: f32, mod_freq: f32, width: f32, num_channels: usize) -> Self {
        let width = (width * sample_rate_hz).round() as usize;
        let mod_freq: f32 = mod_freq / sample_rate_hz;
        let len_samples = 2+width*3;
        let mut filter = Self {
            sample_rate_hz,
            mod_freq,
            width,
            num_channels,
            delay_line: Vec::<RingBuffer<f32>>::new(),
            sample_index: Vec::new()
        };
        for channel in 0..filter.num_channels{
            filter.delay_line.push(RingBuffer::new(len_samples));
            for _ in 0..len_samples { filter.delay_line[channel].push(f32::default()); }
            filter.sample_index.push(1);
        }
        filter
    }
}

impl Processor for Vibrato
{
    type Item = f32;

    fn reset(&mut self) {
        for channel in 0..self.num_channels{
            self.sample_index[channel] = 1;
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
            // ISSUE: len_samples DO NOT match the N-2 criteria in the loop
            for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter_mut()) {
                let modulator = lfo(self.mod_freq, self.sample_index[channel] as f32);
                let offset = 1.0 + self.width as f32 + self.width as f32 * modulator;
                let _ = self.delay_line[channel].pop();
                self.delay_line[channel].push(*input_sample);
                *output_sample = self.delay_line[channel].get_frac(offset);
                self.sample_index[channel]+=1;
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