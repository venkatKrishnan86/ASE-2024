use std::{f32::consts::PI, os};

use nih_plug::params::enums::Enum;

// Premake the LFO sine, and use mod frequency as a phase value to get_frac values
#[derive(Clone, Enum, PartialEq)]
pub enum Oscillator {
    Sine,
    Triangle
}

#[derive(Clone)]
pub struct LFO {
    sample_rate: u32,
    oscillator: Oscillator,
    wave_table_size: usize,
    wave_table: Vec<f32>,
    index: f32,
    index_increment: f32
}

impl LFO {
    pub fn new(sample_rate: u32, wave_table_size: usize, oscillator: Oscillator, frequency: f32) -> Self {
        Self {
            sample_rate,
            oscillator: oscillator.clone(),
            wave_table_size,
            wave_table: LFO::create_wavetable(oscillator, wave_table_size),
            index: 0.0,
            index_increment: frequency * wave_table_size as f32 / sample_rate as f32
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) -> Result<(), String> {
        if frequency < 0.0 {
            return Err("Frequency must be a positive floating point value!".to_owned());
        }
        self.index_increment = frequency * self.wave_table_size as f32 / self.sample_rate as f32;
        Ok(())
    }

    pub fn get_frequency(&self) -> f32 {
        self.index_increment * self.sample_rate as f32 / self.wave_table_size as f32
    }

    pub fn set_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillator = oscillator.clone();
        self.wave_table = LFO::create_wavetable(oscillator, self.wave_table_size)
    }

    fn create_wavetable(oscillator: Oscillator, wave_table_size: usize) -> Vec<f32>{
        let mut wave_table: Vec<f32> = Vec::new();
        match oscillator {
            Oscillator::Sine => {
                for i in 0..wave_table_size {
                    wave_table.push((2.0 * PI * (i as f32)/(wave_table_size as f32)).sin());
                }
            },
            Oscillator::Triangle => {
                for i in 0..wave_table_size/2 {
                    wave_table.push(((i as f32/wave_table_size as f32)*4.0) - 1.0);
                }
                for i in wave_table_size/2..wave_table_size {
                    wave_table.push((-(i as f32/wave_table_size as f32)*4.0) + 3.0);
                }
            }
        }
        wave_table
    }

    pub fn get_sample(&mut self) -> f32 {
        let index_1 = self.index.trunc() as usize;
        let frac = self.index - index_1 as f32;
        self.index = (self.index + self.index_increment) % self.wave_table_size as f32;
        LFO::lerp(self.wave_table[index_1], self.wave_table[(index_1 + 1)%self.wave_table_size], frac)
    }

    pub fn reset(&mut self) {
        self.index = 0.0;
    }

    fn lerp(sample1: f32, sample2: f32, frac: f32) -> f32{
        (1.0-frac)*sample1 + frac*sample2
    }
}