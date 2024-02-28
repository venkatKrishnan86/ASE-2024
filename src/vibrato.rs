use std::f32::consts::PI;

use crate::ring_buffer::RingBuffer;
use crate::utils::{Processor, FilterParam};

pub struct Vibrato<T> 
where T: Copy + Default + Into<f32>
{
    sample_rate_hz: f32,
    mod_freq: f32,
    width: usize,
    num_channels: usize,
    delay_line: Vec<RingBuffer<T>>,
    sample_index: usize
}

impl<T> Vibrato<T>
where T: Copy + Default + Into<f32>
{
    pub fn new(sample_rate_hz: f32, mod_freq: f32, width: T, num_channels: usize) -> Self {
        let width = (width.into() * sample_rate_hz).round() as usize;
        let mod_freq: f32 = mod_freq / sample_rate_hz;
        let len_samples = 2+width*3;
        let mut filter = Self {
            sample_rate_hz,
            mod_freq,
            width,
            num_channels,
            delay_line: Vec::<RingBuffer<T>>::new(),
            sample_index: 1
        };
        for channel in 0..filter.num_channels{
            filter.delay_line.push(RingBuffer::new(len_samples));
            for _ in 0..len_samples { filter.delay_line[channel].push(T::default()); }
        }
        filter
    }
}

impl Processor for Vibrato<f32> 
{
    type Item = f32;

    fn reset(&mut self) {
        self.sample_index = 1;
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
            // ISSUE: len_samples DO NOT match the N-2 criteria in the loop
            for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter_mut()) {
                let modulator = (self.mod_freq * 2.0 * PI * self.sample_index as f32).sin();
                let offset = 1.0 + self.width as f32 + self.width as f32 * modulator;
                let _ = self.delay_line[channel].pop();
                self.delay_line[channel].push(*input_sample);
                *output_sample = self.delay_line[channel].get_frac(offset);
                self.sample_index+=1;
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

// use std::io::{BufReader, BufWriter};
// use std::fs::File;
// use crate::utils;
// use rand;

// use hound::{WavReader, WavSpec, WavWriter};
// use ringbuffer::{AllocRingBuffer, RingBuffer};

// pub struct CombFilter {
//     // TODO: your code here
//     filter_type: FilterType,
//     delay: f32,
//     delayed_signal_amp_factor: f32,
//     sample_rate_hz: f32,
//     num_channels: usize,
//     delay_line: Vec<AllocRingBuffer<f32>>
// }

// #[derive(Debug, Clone, Copy)]
// pub enum FilterType {
//     FIR,
//     IIR,
// }

// #[derive(Debug, Clone, Copy)]
// pub enum FilterParam {
//     Gain,
//     Delay,
// }

// #[derive(Debug, Clone)]
// pub enum Error {
//     InvalidValue { param: FilterParam, value: f32 }
// }

// impl CombFilter {
//     pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
//         let mut filter = Self {
//             filter_type: filter_type,
//             delay: max_delay_secs,
//             delayed_signal_amp_factor: 0.5,
//             sample_rate_hz: sample_rate_hz,
//             num_channels: num_channels,
//             delay_line: Vec::new()
//         };
//         for _ in 0..filter.num_channels{
//             filter.delay_line.push(AllocRingBuffer::with_capacity((max_delay_secs * sample_rate_hz) as usize));
//         }
//         filter.reset();
//         filter
//     }

//     pub fn reset(&mut self) {
//         for i in 0..self.num_channels{
//             self.delay_line[i].clear();
//             for _ in 0..(self.delay*self.sample_rate_hz) as usize {
//                 self.delay_line[i].push(0.0);
//             }
//         }
//     }

//     pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
//         match self.filter_type {
//             FilterType::FIR => {
//                 for (i, (input_channel, output_channel)) in input.iter().zip(output.iter_mut()).enumerate() {
//                     for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter_mut()) {
//                         *output_sample = *input_sample + self.delayed_signal_amp_factor * self.delay_line[i].peek().unwrap_or(&0.0);
//                         self.delay_line[i].dequeue();
//                         self.delay_line[i].push(*input_sample);
//                     }
//                 }
//             },
//             FilterType::IIR => {
//                 for (i, (input_channel, output_channel)) in input.iter().zip(output.iter_mut()).enumerate() {
//                     for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter_mut()) {
//                         *output_sample = *input_sample + self.delayed_signal_amp_factor * self.delay_line[i].peek().unwrap_or(&0.0);
//                         self.delay_line[i].dequeue();
//                         self.delay_line[i].push(*output_sample);
//                     }
//                 }
//             }
//         }
//     }

//     pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
//         match param {
//             FilterParam::Gain => {
//                 if value < 0.0 || value > 1.0 {
//                     return Err(Error::InvalidValue { param: param, value: value });
//                 }
//                 self.delayed_signal_amp_factor = value
//             },
//             FilterParam::Delay => {
//                 if value > self.delay_line.get(0).expect("Missing delay line").capacity() as f32/self.sample_rate_hz || value < 0.0 {
//                     return Err(Error::InvalidValue { param: param, value: value });
//                 }
//                 self.delay = value;
//                 self.reset();
//             }
//         }
//         Ok(())
//     }

//     pub fn get_param(&self, param: FilterParam) -> f32 {
//         match param {
//             FilterParam::Gain => self.delayed_signal_amp_factor,
//             FilterParam::Delay => self.delay
//         }
//     }
    
//     // TODO: feel free to define other functions for your own use
// }

// // TODO: feel free to define other types (here or in other modules) for your own use
// pub fn process_and_write_audio(
//     reader: &mut WavReader<BufReader<File>>, 
//     block_size: usize, 
//     channels: usize, 
//     output_file: &String, 
//     spec: WavSpec, 
//     filter_type: FilterType,
//     gain: f32,
//     max_delay_secs: f32
// ){

//     let mut comb_filter_1: CombFilter = CombFilter::new(filter_type, max_delay_secs, spec.sample_rate as f32, channels);
//     comb_filter_1.set_param(FilterParam::Gain, gain).expect("Incorrect gain value");
//     let mut writer: WavWriter<BufWriter<File>> = WavWriter::create(output_file, spec).expect("Unable to create file");

//     while let Ok(block) = reader.samples::<i16>().take(block_size*channels).collect::<Result<Vec<_>, _>>() {
//         let mut process_block = utils::ProcessBlocks::new(&block, &channels);
//         let (input_address, mut output_address) = process_block.create_and_write_addresses();
//         comb_filter_1.process(&input_address, &mut output_address);
//         process_block.write_output_samples(&mut writer).unwrap();
//         if block.len() < block_size*channels as usize { break }
//     }
// }