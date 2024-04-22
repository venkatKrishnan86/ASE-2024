use hound::WavWriter;
use std::{fs::File, io::BufWriter};

#[allow(dead_code)]
pub enum FilterParam {
    ModFreq,
    Width
}

pub trait Processor {
    type Item;

    fn reset(&mut self);
    fn process(&mut self, input: &[&[Self::Item]], output: &mut[&mut[Self::Item]]);
    fn get_param(&self, param: FilterParam) -> Self::Item;
    fn set_param(&mut self, param: FilterParam, value: Self::Item) -> Result<(), String>;
}

pub struct ProcessBlocks {
    pub input_block: Vec<f32>,
    pub output_block: Vec<f32>
}

impl ProcessBlocks {
    pub fn new(audio: &[i16]) -> Self {
        let length = audio.len();
        let mut block = Self {
            input_block: vec![0.0; length],
            output_block: vec![0.0; length]
        };
        block.convert_i16_samples_to_f32(audio);
        block
    }

    fn convert_i16_samples_to_f32(&mut self, audio: &[i16]) {
        for (i, &sample) in audio.iter().enumerate() {
            let sample = i16_to_f32(sample);
            self.input_block[i] = sample;
        }
    }

    pub fn get_addresses(&mut self) -> (&[f32], &mut [f32]) {
        (&self.input_block, &mut self.output_block)
    }

    #[allow(dead_code)]
    pub fn write_output_samples(&mut self, writer: &mut WavWriter<BufWriter<File>>) -> Result<(), hound::Error> {
        let new_output_block = transpose(vec![self.output_block.clone()]);

        for output in new_output_block.into_iter() {
            for value in output {
                writer.write_sample(f32_to_i16(value))?;
            }
        }
        Ok(())
    }
}

pub fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

pub fn f32_to_i16(value: f32) -> i16 {
    // assert!(value>=-1.0 && value<1.0);
    (value*(-(i16::MIN as f32))) as i16
}

pub fn i16_to_f32(value: i16) -> f32 {
    value as f32 / (1 << 15) as f32
}

#[allow(dead_code)]
pub fn is_close(a: f32, b: f32, rel_close: f32) -> bool {
    (a-b).abs() < rel_close
}