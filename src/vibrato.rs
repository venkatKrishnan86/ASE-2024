use crate::ring_buffer::RingBuffer;
use crate::utils::{Processor, FilterParam};
use crate::lfo;

/// # Vibrato Filter
///
/// This contains the -
/// 1. Sample rate of the audio (Hz)
/// 2. Width parameter (seconds)
/// 3. Number of channels
/// 3. The ModFreq parameter (Hz) is used to create the LFO wavetable
/// 4. Delay Line, as a vector of ring buffers, of size `num_channels`
///
/// It implements the trait `Processor`, and processes the audio using the `process` function
/// Choosing these units for sample rate, mod_freq, and width is purely based on the most common choices for these three (Hz, Hz and seconds)
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
    /// Constructor takes in the 
    ///
    /// ## Arguments
    /// `sample_rate_hz: f32`: The sample rate of the audio in Hz
    /// `mod_freq: f32` : The modulation frequency parameter value in Hz
    /// `width: f32`:  The width parameter value in seconds
    /// `num_channels: usize`: Number of channels
    ///
    /// ## Returns
    ///
    /// `Vibrato`: Creates the vibrato object 
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
    use crate::utils::{ProcessBlocks, is_close}; 

    mod set_param_tests {
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
        use rand::random;

        #[test]
        fn test_1_zero_input() {
            let num_channels = (random::<f32>()*6.0) as usize + 1;
            let mod_freq = (random::<f32>()+f32::EPSILON)*20.0;
            let width = (random::<f32>()+f32::EPSILON)*10.0;
            let mut filter = Vibrato::new(44100.0, mod_freq, width, num_channels);

            let mut process_block = ProcessBlocks::new(&vec![0; 6*5*4*3*2*1], &num_channels);
            process_block.input_block = vec![vec![0.0; 6*5*4*3*2*1]; num_channels];
            process_block.output_block = vec![vec![0.0; 6*5*4*3*2*1]; num_channels];

            let (input_address, mut output_address) = process_block.create_and_write_addresses();

            filter.process(&input_address, &mut output_address);

            for channel in process_block.output_block {
                for value in channel {
                    assert!(is_close(0.0, value, 0.0001));
                }
            }
        }

        #[test]
        fn test_2_dc_input() {
            // Upto 6 channels
            let num_channels = (random::<f32>()*6.0) as usize + 1;
            let mod_freq = (random::<f32>()+f32::EPSILON)*20.0;
            let width = (random::<f32>()+f32::EPSILON)*20.0;
            let mut filter = Vibrato::new(1000.0, mod_freq, width, num_channels);

            let mut process_block = ProcessBlocks::new(&vec![i16::MAX; 5*4*3*2*1], &num_channels);
            process_block.input_block = vec![vec![1.0; (5*4*3*2*1)/num_channels]; num_channels];
            process_block.output_block = vec![vec![0.0; (5*4*3*2*1)/num_channels]; num_channels];

            let (input_address, mut output_address) = process_block.create_and_write_addresses();

            filter.process(&input_address, &mut output_address);

            for channel in process_block.output_block {
                for value in channel {
                    assert!(is_close(0.0, value, 0.0001), "{value}");
                }
            }
        }

        fn test_3_mono_blocking() {
            let block_size = 4;
            let channels = 1;
        
            let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
            let mut samples_address = &samples[..];

            let mut iteration_num = 1;

            while !samples_address.is_empty() {
                let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));

                let process_block = ProcessBlocks::new(&block.to_vec(), &channels);

                if iteration_num == 1 {
                    assert_eq!(process_block.input_block, vec![vec![4.27246094e-04, 1.31958008e-01, -1.04431152e-01, 3.06152344e-01]]);
                } else if iteration_num == 2 {
                    assert_eq!(process_block.input_block, vec![vec![9.89746094e-01, -7.01904297e-04, -7.14721680e-02, -9.15527344e-05]]);
                } else if iteration_num == 3 {
                    assert_eq!(process_block.input_block, vec![vec![3.12194824e-02, 7.04956055e-03]]);
                } else {
                    panic!("Taking extra loops unnecessarily");
                }

                samples_address = rest;
                iteration_num += 1;
            }
        }

        #[test]
        fn test_3_stereo_blocking() {
            let block_size = 2;
            let channels = 2;
            
            let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
            let mut samples_address = &samples[..];

            let mut iteration_num = 1;

            while !samples_address.is_empty() {
                let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));

                let process_block = ProcessBlocks::new(&block.to_vec(), &channels);

                if iteration_num == 1 {
                    assert_eq!(process_block.input_block, vec![vec![4.27246094e-04, -1.04431152e-01], vec![1.31958008e-01, 3.06152344e-01]])
                } else if iteration_num == 2 {
                    assert_eq!(process_block.input_block, vec![vec![9.89746094e-01, -7.14721680e-02], vec![-7.01904297e-04, -9.15527344e-05]])
                } else if iteration_num == 3 {
                    assert_eq!(process_block.input_block, vec![vec![3.12194824e-02], vec![7.04956055e-03]])
                } else {
                    panic!("Taking extra loops unnecessarily");
                }

                samples_address = rest;
                iteration_num += 1;
            }
        }
    }
}

//     mod audio_larger_than_buffer_size_tests {
//         use super::*;

//         /// Must pass test_mono_channel_blocking();
//         #[test]
//         fn test_mono_process_iir_impulse_for_larger_than_buffer_size_audio() {
//             let block_size = 2;
//             let channels = 1;
//             let mut filter = CombFilter::new(FilterType::IIR, 0.2, 10.0, channels);
            
//             let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
//             let mut samples_address = &samples[..];

//             let mut final_output: Vec<f32> = Vec::new();

//             while !samples_address.is_empty() {
//                 let (block, rest) = samples_address.split_at(std::cmp::min(block_size, samples_address.len()));
//                 let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
//                 let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                 filter.process(&input_address, &mut output_address);

//                 for channel in process_block.output_block {
//                     for i in channel{
//                         final_output.push(i);
//                     }
//                 }
//                 samples_address = rest;
//             }
//             let actual_vector: Vec<f32> = vec![
//                 4.27246094e-04,
//                 1.31958008e-01,
//                 -1.04217529e-01,
//                 3.72131348e-01,
//                 9.37637329e-01,
//                 1.85363770e-01,
//                 3.97346497e-01,
//                 9.25903320e-02,
//                 2.29892731e-01,
//                 5.33447266e-02
//             ];
//             for (i,j) in actual_vector.into_iter().zip(final_output.into_iter()) {
//                 assert!(is_close(i, j));
//             }
//             // assert_eq!(actual_vector, final_output);
//         }

//         /// Must pass test_mono_channel_blocking();
//         #[test]
//         fn test_mono_process_fir_impulse_for_larger_than_buffer_size_audio() {
//             let block_size = 2;
//             let channels = 1;
//             let mut filter = CombFilter::new(FilterType::FIR, 0.2, 10.0, channels);
            
//             let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
//             let mut samples_address = &samples[..];

//             let mut final_output: Vec<f32> = Vec::new();

//             while !samples_address.is_empty() {
//                 let (block, rest) = samples_address.split_at(std::cmp::min(block_size, samples_address.len()));

//                 let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
//                 let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                 filter.process(&input_address, &mut output_address);
//                 for channel in process_block.output_block {
//                     for i in channel{
//                         final_output.push(i);
//                     }
//                 }

//                 samples_address = rest;
//             }
//             let actual_vector: Vec<f32> = vec![
//                 4.27246094e-04,
//                 1.31958008e-01,
//                 -1.04217529e-01,
//                 3.72131348e-01,
//                 9.37530518e-01,
//                 1.52374268e-01,
//                 4.23400879e-01,
//                 -4.42504883e-04,
//                 -4.51660156e-03,
//                 7.00378418e-03
//             ];
//             for (i,j) in actual_vector.into_iter().zip(final_output.into_iter()) {
//                 assert!(is_close(i, j));
//             }
//             // assert_eq!(actual_vector, final_output);
//         }

//         /// Must pass `test_stereo_channel_blocking()`;
//         #[test]
//         fn test_stereo_process_iir_impulse_for_larger_than_buffer_size_audio() {
//             let block_size = 2;
//             let channels = 2;
//             let mut filter = CombFilter::new(FilterType::IIR, 0.2, 5.0, channels);
            
//             let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
//             let mut samples_address = &samples[..];

//             let mut final_output_left: Vec<f32> = Vec::new();
//             let mut final_output_right: Vec<f32> = Vec::new();

//             while !samples_address.is_empty() {
//                 let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));

//                 let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);

//                 let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                 filter.process(&input_address, &mut output_address);
                
//                 for channel in process_block.output_block[0].clone() {
//                     final_output_left.push(channel);
//                 }
//                 for channel in process_block.output_block[1].clone() {
//                     final_output_right.push(channel);
//                 }

//                 samples_address = rest;
//             }
//             let actual_left: Vec<f32> = vec![4.27246094e-04, -1.04217529e-01, 9.37637329e-01, 3.97346497e-01, 2.29892731e-01];
//             let actual_right: Vec<f32> = vec![0.13195801, 0.37213135, 0.18536377, 0.09259033, 0.05334473];
//             for (i,j) in actual_left.into_iter().zip(final_output_left.into_iter()) {
//                 assert!(is_close(i, j));
//             }
//             for (i,j) in actual_right.into_iter().zip(final_output_right.into_iter()) {
//                 assert!(is_close(i, j));
//             }
//             // assert_eq!(actual_left, final_output_left);
//             // assert_eq!(actual_right, final_output_right);
//         }

//         /// Must pass `test_stereo_channel_blocking()`;
//         #[test]
//         fn test_stereo_process_fir_impulse_for_larger_than_buffer_size_audio() {
            
//             let block_size = 2;
//             let channels = 2;
//             let mut filter = CombFilter::new(FilterType::FIR, 0.2, 5.0, channels);
            
//             let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
//             let mut samples_address = &samples[..];

//             let mut final_output_left: Vec<f32> = Vec::new();
//             let mut final_output_right: Vec<f32> = Vec::new();

//             while !samples_address.is_empty() {
//                 let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));

//                 let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);

//                 let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                 filter.process(&input_address, &mut output_address);
                
//                 for channel in process_block.output_block[0].clone() {
//                     final_output_left.push(channel);
//                 }
//                 for channel in process_block.output_block[1].clone() {
//                     final_output_right.push(channel);
//                 }

//                 samples_address = rest;
//             }
//             let actual_left: Vec<f32> = vec![4.27246094e-04, -1.04217529e-01,  9.37530518e-01,  4.23400879e-01, -4.51660156e-03];
//             let actual_right: Vec<f32> = vec![0.13195801, 0.37213135, 0.15237427, -0.0004425, 0.00700378];
//             for (i,j) in actual_left.into_iter().zip(final_output_left.into_iter()) {
//                 assert!(is_close(i, j));
//             }
//             for (i,j) in actual_right.into_iter().zip(final_output_right.into_iter()) {
//                 assert!(is_close(i, j));
//             }
//             // assert_eq!(actual_left, final_output_left);
//             // assert_eq!(actual_right, final_output_right);
//         }

//         /// TEST: A random signal must be processed the exact same way, and must yield the exact same results
//         /// These tests within this `mod` achieves that
//         mod different_buffer_sizes_and_channels_tests {
//             use super::*;

//             /// Channels: Mono
//             /// Filter Type: IIR
//             #[test]
//             fn different_buffer_sizes_mono_iir_test() {
//                 let channels: usize = 1;
//                 let buffer_sizes: [usize; 6] = [32, 64, 128, 256, 512, 1024];
//                 let mut final_output: Vec<Vec<f32>> = vec![Vec::new(); 6];

//                 let mut input: [i16; 3000] = [0; 3000];
//                 for i in &mut input {
//                     *i = utils::f32_to_i16((rand::random::<f32>()*2.0) - 1.0);
//                 }

//                 for (i, block_size) in buffer_sizes.iter().enumerate() {
//                     let mut filter = CombFilter::new(FilterType::IIR, 0.5, 100.0, channels);
//                     let temp_input = input.clone();
//                     let mut samples_address = &temp_input[..];
//                     while !samples_address.is_empty() {
//                         let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));
//                         let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
        
//                         let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                         filter.process(&input_address, &mut output_address);
                        
//                         for values in process_block.output_block[0].clone() {
//                             final_output[i].push(values);
//                         }
//                         samples_address = rest;
//                     }
//                 }
//                 for i in 0..5 {
//                     for (value_1, value_2) in final_output[i].iter().zip(final_output[i+1].iter()) {
//                         assert!(is_close(*value_1, *value_2));
//                     }
//                 }
//             }

//             /// Channels: Mono
//             /// Filter Type: FIR
//             #[test]
//             fn different_buffer_sizes_mono_fir_test() {
//                 let channels: usize = 1;
//                 let buffer_sizes: [usize; 6] = [32, 64, 128, 256, 512, 1024];
//                 let mut final_output: Vec<Vec<f32>> = vec![Vec::new(); 6];

//                 let mut input: [i16; 3000] = [0; 3000];
//                 for i in &mut input {
//                     *i = utils::f32_to_i16((rand::random::<f32>()*2.0) - 1.0);
//                 }

//                 for (i, block_size) in buffer_sizes.iter().enumerate() {
//                     let mut filter = CombFilter::new(FilterType::FIR, 0.5, 100.0, channels);
//                     let temp_input = input.clone();
//                     let mut samples_address = &temp_input[..];
//                     while !samples_address.is_empty() {
//                         let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));
//                         let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
        
//                         let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                         filter.process(&input_address, &mut output_address);
                        
//                         for values in process_block.output_block[0].clone() {
//                             final_output[i].push(values);
//                         }
//                         samples_address = rest;
//                     }
//                 }
//                 for i in 0..5 {
//                     for (value_1, value_2) in final_output[i].iter().zip(final_output[i+1].iter()) {
//                         assert!(is_close(*value_1, *value_2));
//                     }
//                 }
//             }

//             /// Channels: Stereo
//             /// Filter Type: IIR
//             #[test]
//             fn different_buffer_sizes_stereo_iir_test() {
//                 let channels: usize = 2;
//                 let buffer_sizes: [usize; 6] = [32, 64, 128, 256, 512, 1024];
//                 let mut final_output_left: Vec<Vec<f32>> = vec![Vec::new(); 6];
//                 let mut final_output_right: Vec<Vec<f32>> = vec![Vec::new(); 6];

//                 let mut input: [i16; 3000] = [0; 3000];
//                 for i in &mut input {
//                     *i = utils::f32_to_i16((rand::random::<f32>()*2.0) - 1.0);
//                 }

//                 for (i, block_size) in buffer_sizes.iter().enumerate() {
//                     let mut filter = CombFilter::new(FilterType::IIR, 0.5, 100.0, channels);
//                     let temp_input = input.clone();
//                     let mut samples_address = &temp_input[..];
//                     while !samples_address.is_empty() {
//                         let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));
//                         let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
        
//                         let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                         filter.process(&input_address, &mut output_address);
                        
//                         for values in process_block.output_block[0].clone() {
//                             final_output_left[i].push(values);
//                         }
//                         for values in process_block.output_block[1].clone() {
//                             final_output_right[i].push(values);
//                         }
//                         samples_address = rest;
//                     }
//                 }
//                 for i in 0..5 {
//                     for (value_1, value_2) in final_output_left[i].iter().zip(final_output_left[i+1].iter()) {
//                         assert!(is_close(*value_1, *value_2));
//                     }
//                     for (value_1, value_2) in final_output_right[i].iter().zip(final_output_right[i+1].iter()) {
//                         assert!(is_close(*value_1, *value_2));
//                     }
//                 }
//             }

//             /// Channels: Stereo
//             /// Filter Type: FIR
//             #[test]
//             fn different_buffer_sizes_stereo_fir_test() {
//                 let channels: usize = 2;
//                 let buffer_sizes: [usize; 6] = [32, 64, 128, 256, 512, 1024];
//                 let mut final_output_left: Vec<Vec<f32>> = vec![Vec::new(); 6];
//                 let mut final_output_right: Vec<Vec<f32>> = vec![Vec::new(); 6];

//                 let mut input: [i16; 3000] = [0; 3000];
//                 for i in &mut input {
//                     *i = utils::f32_to_i16((rand::random::<f32>()*2.0) - 1.0);
//                 }

//                 for (i, block_size) in buffer_sizes.iter().enumerate() {
//                     let mut filter = CombFilter::new(FilterType::FIR, 0.5, 100.0, channels);
//                     let temp_input = input.clone();
//                     let mut samples_address = &temp_input[..];
//                     while !samples_address.is_empty() {
//                         let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));
//                         let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
        
//                         let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                         filter.process(&input_address, &mut output_address);
                        
//                         for values in process_block.output_block[0].clone() {
//                             final_output_left[i].push(values);
//                         }
//                         for values in process_block.output_block[1].clone() {
//                             final_output_right[i].push(values);
//                         }
//                         samples_address = rest;
//                     }
//                 }
//                 for i in 0..5 {
//                     for (value_1, value_2) in final_output_left[i].iter().zip(final_output_left[i+1].iter()) {
//                         assert!(is_close(*value_1, *value_2));
//                     }
//                     for (value_1, value_2) in final_output_right[i].iter().zip(final_output_right[i+1].iter()) {
//                         assert!(is_close(*value_1, *value_2));
//                     }
//                 }
//             }

//             /// Channels: Spatial (5 channel)
//             /// Filter Type: IIR
//             #[test]
//             fn different_buffer_sizes_spatial_iir_test() {
//                 let channels: usize = 5;
//                 let buffer_sizes: [usize; 6] = [32, 64, 128, 256, 512, 1024];
//                 let mut final_output_spatial: Vec<Vec<Vec<f32>>> = vec![vec![Vec::new(); 6]; channels];

//                 let mut input: [i16; 3000] = [0; 3000];
//                 for i in &mut input {
//                     *i = utils::f32_to_i16((rand::random::<f32>()*2.0) - 1.0);
//                 }

//                 for (i, block_size) in buffer_sizes.iter().enumerate() {
//                     let mut filter = CombFilter::new(FilterType::IIR, 0.5, 100.0, channels);
//                     let temp_input = input.clone();
//                     let mut samples_address = &temp_input[..];
//                     while !samples_address.is_empty() {
//                         let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));
//                         let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
        
//                         let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                         filter.process(&input_address, &mut output_address);
                        
//                         for (channel_num, final_output_channel) in final_output_spatial.iter_mut().enumerate() {
//                             for values in process_block.output_block[channel_num].clone() {
//                                 final_output_channel[i].push(values);
//                             }
//                         }
//                         samples_address = rest;
//                     }
//                 }
//                 for i in 0..5 {
//                     for final_output_channel in final_output_spatial.iter() {
//                         for (value_1, value_2) in final_output_channel[i].iter().zip(final_output_channel[i+1].iter()) {
//                             assert!(is_close(*value_1, *value_2));
//                         }
//                     }
//                 }
//             }

//             /// Channels: Spatial (5 channel)
//             /// Filter Type: FIR
//             #[test]
//             fn different_buffer_sizes_spatial_fir_test() {
//                 let channels: usize = 5;
//                 let buffer_sizes: [usize; 6] = [32, 64, 128, 256, 512, 1024];
//                 let mut final_output_spatial: Vec<Vec<Vec<f32>>> = vec![vec![Vec::new(); 6]; channels];

//                 let mut input: [i16; 3000] = [0; 3000];
//                 for i in &mut input {
//                     *i = utils::f32_to_i16((rand::random::<f32>()*2.0) - 1.0);
//                 }

//                 for (i, block_size) in buffer_sizes.iter().enumerate() {
//                     let mut filter = CombFilter::new(FilterType::FIR, 0.5, 100.0, channels);
//                     let temp_input = input.clone();
//                     let mut samples_address = &temp_input[..];
//                     while !samples_address.is_empty() {
//                         let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));
//                         let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
        
//                         let (input_address, mut output_address) = process_block.create_and_write_addresses();
//                         filter.process(&input_address, &mut output_address);
                        
//                         for (channel_num, final_output_channel) in final_output_spatial.iter_mut().enumerate() {
//                             for values in process_block.output_block[channel_num].clone() {
//                                 final_output_channel[i].push(values);
//                             }
//                         }
//                         samples_address = rest;
//                     }
//                 }
//                 for i in 0..5 {
//                     for final_output_channel in final_output_spatial.iter() {
//                         for (value_1, value_2) in final_output_channel[i].iter().zip(final_output_channel[i+1].iter()) {
//                             assert!(is_close(*value_1, *value_2));
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     #[test]
//     fn test_process_fir_impulse_rand_x_mono() {
//         let mut filter = CombFilter::new(FilterType::FIR, 0.5, 10.0, 1);
//         filter.set_param(FilterParam::Delay, 0.2).unwrap();
        
//         let mut process_block = ProcessBlocks::new(&vec![0; 10], &1);
//         process_block.input_block = vec![vec![
//             0.35718214,
//             0.71631462,
//             0.17056465,
//             0.00772361,
//             0.29818118,
//             0.197505,
//             0.76061135,
//             0.08108575,
//             0.64278864,
//             0.93702414
//         ]];
//         process_block.output_block = vec![vec![0.0; 10]];

//         let (input_address, mut output_address) = process_block.create_and_write_addresses();

//         filter.process(&input_address, &mut output_address);

//         // Checked on python implementation
//         let actual_value: [f32; 10] = [0.35718214, 0.71631462, 0.34915572, 0.36588091, 0.3834635, 0.2013668 , 0.90970194, 0.17983825, 1.02309432, 0.97756701];
//         let final_output = process_block.output_block[0].clone();
//         for (a,b) in final_output.into_iter().zip(actual_value){
//             assert!(utils::is_close(a, b));
//         }
//     }

//     #[test]
//     fn test_process_iir_impulse_rand_x_mono() {
//         let mut filter = CombFilter::new(FilterType::IIR, 0.5, 10.0, 1);
//         filter.set_param(FilterParam::Delay, 0.2).unwrap();

//         let mut process_block = ProcessBlocks::new(&vec![0; 10], &1);
//         process_block.input_block = vec![vec![
//             0.35718214,
//             0.71631462,
//             0.17056465,
//             0.00772361,
//             0.29818118,
//             0.197505,
//             0.76061135,
//             0.08108575,
//             0.64278864,
//             0.93702414
//         ]];
//         process_block.output_block = vec![vec![0.0; 10]];

//         let (input_address, mut output_address) = process_block.create_and_write_addresses();

//         filter.process(&input_address, &mut output_address);

//         // Checked on python implementation
//         let actual_value: [f32; 10] = [0.35718214, 0.71631462, 0.34915572, 0.36588091, 0.47275904, 0.38044546, 0.99699087, 0.27130848, 1.14128408, 1.07267838];
//         let final_output = process_block.output_block[0].clone();
//         for (a,b) in final_output.into_iter().zip(actual_value){
//             assert!(utils::is_close(a, b));
//         }
//     }

//     #[test]
//     fn zero_output_input_freq_matching_feedforward() {
//         let mut v: Vec<f32> = Vec::new();
//         let num_samples = 100;
//         let freq = 10.0;
//         for i in 0..num_samples {
//             v.push(f32::sin(2.0*PI*(i as f32/num_samples as f32) * freq));
//         }

//         let mut filter = CombFilter::new(FilterType::FIR, 1.0/(2.0*freq), num_samples as f32, 1);
//         filter.set_param(FilterParam::Gain, 1.0).unwrap();
        
//         let mut process_block = ProcessBlocks::new(&vec![0; num_samples], &1);
//         process_block.input_block = vec![v];
//         process_block.output_block = vec![vec![0.0; num_samples]];

//         let (input_address, mut output_address) = process_block.create_and_write_addresses();

//         filter.process(&input_address, &mut output_address);

//         let output_array = process_block.output_block[0].clone();

//         // Checking for 0s after the half frequency point
//         for value in output_array[(freq/2.0) as usize..].iter() {
//             assert!(f32::abs(*value) < 1e-4); // Since the subtraction is not accurate
//         }
//     }

//     #[test]
//     fn iir_magnitude_increase_for_input_freq_matching_feedforward() {
//         let mut v: Vec<f32> = Vec::new();
//         let num_samples = 1000;
//         let freq = 100.0;
//         for i in 0..num_samples {
//             v.push(f32::sin(2.0*PI*(i as f32/num_samples as f32) * freq));
//         }

//         let mut filter = CombFilter::new(FilterType::IIR, 1.0/(freq), num_samples as f32, 1);
//         filter.set_param(FilterParam::Gain, 1.0).unwrap();
        
//         let mut process_block = ProcessBlocks::new(&vec![0; num_samples], &1);
//         process_block.input_block = vec![v];
//         process_block.output_block = vec![vec![0.0; num_samples]];

//         let (input_address, mut output_address) = process_block.create_and_write_addresses();

//         filter.process(&input_address, &mut output_address);

//         let output_array = process_block.output_block[0].clone();

//         let final_i16_output: Vec<i16> = output_array.iter().map(|value| f32_to_i16(*value)).collect();
//         assert_eq!(i16::MIN, *final_i16_output.iter().min().unwrap_or(&0));
//         assert_eq!(i16::MAX, *final_i16_output.iter().max().unwrap_or(&0));
//     }
// }