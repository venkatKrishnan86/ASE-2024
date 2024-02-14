use std::io::{BufReader, BufWriter};
use std::fs::File;
use crate::utils;

use hound::{WavReader, WavSpec, WavWriter};
use ringbuffer::{AllocRingBuffer, RingBuffer};

pub struct CombFilter {
    // TODO: your code here
    filter_type: FilterType,
    delay: f32,
    delayed_signal_amp_factor: f32,
    sample_rate_hz: f32,
    num_channels: usize,
    delay_line: Vec<AllocRingBuffer<f32>>
}

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 }
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        let mut filter = Self {
            filter_type: filter_type,
            delay: max_delay_secs,
            delayed_signal_amp_factor: 0.5,
            sample_rate_hz: sample_rate_hz,
            num_channels: num_channels,
            delay_line: Vec::new()
        };
        for _ in 0..filter.num_channels{
            filter.delay_line.push(AllocRingBuffer::with_capacity((max_delay_secs * sample_rate_hz) as usize));
        }
        filter.reset();
        filter
    }

    pub fn reset(&mut self) {
        for i in 0..self.num_channels{
            self.delay_line[i].clear();
            for _ in 0..(self.delay*self.sample_rate_hz) as usize {
                self.delay_line[i].push(0.0);
            }
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        match self.filter_type {
            FilterType::FIR => {
                for (i, (input_channel, output_channel)) in input.iter().zip(output.iter_mut()).enumerate() {
                    for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter_mut()) {
                        *output_sample = *input_sample + self.delayed_signal_amp_factor * self.delay_line[i].peek().unwrap_or(&0.0);
                        self.delay_line[i].dequeue();
                        self.delay_line[i].push(*input_sample);
                    }
                }
            },
            FilterType::IIR => {
                for (i, (input_channel, output_channel)) in input.iter().zip(output.iter_mut()).enumerate() {
                    for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter_mut()) {
                        *output_sample = *input_sample + self.delayed_signal_amp_factor * self.delay_line[i].peek().unwrap_or(&0.0);
                        self.delay_line[i].dequeue();
                        self.delay_line[i].push(*output_sample);
                    }
                }
            }
        }
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            FilterParam::Gain => {
                if value < 0.0 || value > 1.0 {
                    return Err(Error::InvalidValue { param: param, value: value });
                }
                self.delayed_signal_amp_factor = value
            },
            FilterParam::Delay => {
                if value > self.delay_line.get(0).expect("Missing delay line").capacity() as f32/self.sample_rate_hz || value < 0.0 {
                    return Err(Error::InvalidValue { param: param, value: value });
                }
                self.delay = value;
                self.reset();
            }
        }
        Ok(())
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {
            FilterParam::Gain => self.delayed_signal_amp_factor,
            FilterParam::Delay => self.delay
        }
    }
    
    // TODO: feel free to define other functions for your own use
}

// TODO: feel free to define other types (here or in other modules) for your own use
pub fn process_and_write_audio(reader: &mut WavReader<BufReader<File>>, block_size: usize, channels: usize, output_file: &String, spec: WavSpec) {
    let mut comb_filter_1 = CombFilter::new(FilterType::IIR, 0.1, spec.sample_rate as f32, channels);
    let mut writer: WavWriter<BufWriter<File>> = WavWriter::create(output_file, spec).expect("Unable to create file");

    while let Ok(block) = reader.samples::<i16>().take(block_size*channels).collect::<Result<Vec<_>, _>>() {
        let mut process_block = utils::ProcessBlocks::new(&block, &channels);
        let (input_address, mut output_address) = process_block.create_and_write_addresses();
        comb_filter_1.process(&input_address, &mut output_address);
        process_block.write_output_samples(&mut writer).unwrap();
        if block.len() < block_size*channels as usize { break }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::utils::{self, is_close, ProcessBlocks};

    #[should_panic]
    #[test]
    fn setting_wrong_gain_params_for_comb_filter() {
        let mut filter = CombFilter::new(FilterType::FIR, 0.1, 100.0, 2);
        let result = filter.set_param(FilterParam::Gain, 2.0);
        match result {
            Ok(()) => (),
            Err(_) => panic!("Error")
        }

        let result = filter.set_param(FilterParam::Gain, -0.2);
        match result {
            Ok(()) => (),
            Err(_) => panic!("Error")
        }
    }

    #[test]
    fn setting_correct_gain_params_for_comb_filter() {
        let mut filter = CombFilter::new(FilterType::FIR, 0.1, 100.0, 2);
        let result = filter.set_param(FilterParam::Gain, 0.2);
        match result {
            Ok(()) => (),
            Err(_) => panic!("Error")
        }
    }

    #[test]
    #[should_panic]
    fn setting_negative_delay_params_for_comb_filter() {
        let mut filter = CombFilter::new(FilterType::FIR, 0.1, 100.0, 2);
        let result = filter.set_param(FilterParam::Delay, -1.2);
        match result {
            Ok(()) => (),
            Err(_) => panic!("Error")
        }
    }

    #[test]
    fn setting_delay_params_for_comb_filter() {
        let mut filter = CombFilter::new(FilterType::FIR, 0.1, 100.0, 2);

        let result = filter.set_param(FilterParam::Delay, 0.02);
        match result {
            Ok(()) => (),
            Err(_) => panic!("Error")
        }
    }

    #[test]
    #[should_panic]
    fn setting_larger_delay_params_for_comb_filter() {
        let mut filter = CombFilter::new(FilterType::FIR, 0.1, 100.0, 2);

        let result = filter.set_param(FilterParam::Delay, 1.2);
        match result {
            Ok(()) => (),
            Err(_) => panic!("Error")
        }
    }

    #[test]
    fn test_process_fir_impulse() {
        let mut filter = CombFilter::new(FilterType::FIR, 0.5, 10.0, 1);
        
        let mut process_block = ProcessBlocks::new(&vec![0; 10], &1);
        process_block.input_block = vec![vec![0.0; 10]];
        process_block.output_block = vec![vec![0.0; 10]];
        process_block.input_block[0][0] = 1.0;

        let (input_address, mut output_address) = process_block.create_and_write_addresses();

        filter.process(&input_address, &mut output_address);

        // Checked on python implementation
        assert_eq!(vec![vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0]], process_block.output_block);
    }

    #[test]
    fn test_process_fir_impulse_modified_delay() {
        let mut filter = CombFilter::new(FilterType::FIR, 0.5, 10.0, 1);
        filter.set_param(FilterParam::Delay, 0.3).unwrap();
        
        let mut process_block = ProcessBlocks::new(&vec![0; 10], &1);
        process_block.input_block = vec![vec![0.0; 10]];
        process_block.output_block = vec![vec![0.0; 10]];
        process_block.input_block[0][0] = 1.0;

        let (input_address, mut output_address) = process_block.create_and_write_addresses();

        filter.process(&input_address, &mut output_address);

        // Checked on python implementation
        assert_eq!(vec![vec![1.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]], process_block.output_block);
    }

    #[test]
    fn test_process_iir_impulse() {
        let mut filter = CombFilter::new(FilterType::IIR, 0.5, 10.0, 1);
        
        let mut process_block = ProcessBlocks::new(&vec![0; 10], &1);
        process_block.input_block = vec![vec![0.0; 10]];
        process_block.output_block = vec![vec![0.0; 10]];
        process_block.input_block[0][0] = 1.0;

        let (input_address, mut output_address) = process_block.create_and_write_addresses();

        filter.process(&input_address, &mut output_address);

        // Checked on python implementation
        assert_eq!(vec![vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0]], process_block.output_block);
    }

    #[test]
    fn test_process_iir_impulse_modified_delay() {
        let mut filter = CombFilter::new(FilterType::IIR, 0.5, 10.0, 1);
        filter.set_param(FilterParam::Delay, 0.3).unwrap();
        
        let mut process_block = ProcessBlocks::new(&vec![0; 10], &1);
        process_block.input_block = vec![vec![0.0; 10]];
        process_block.output_block = vec![vec![0.0; 10]];
        process_block.input_block[0][0] = 1.0;

        let (input_address, mut output_address) = process_block.create_and_write_addresses();

        filter.process(&input_address, &mut output_address);

        // Checked on python implementation
        assert_eq!(vec![vec![1.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.25, 0.0, 0.0, 0.125]], process_block.output_block);
    }

    mod larger_than_buffer_size_tests {
        use super::*;

        #[test]
        fn test_mono_channel_blocking() {
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
        fn test_stereo_channel_blocking() {
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

        /// Must pass test_mono_channel_blocking();
        #[test]
        fn test_mono_process_iir_impulse_for_larger_than_buffer_size_audio() {
            let block_size = 2;
            let channels = 1;
            let mut filter = CombFilter::new(FilterType::IIR, 0.2, 10.0, channels);
            
            let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
            let mut samples_address = &samples[..];

            let mut final_output: Vec<f32> = Vec::new();

            while !samples_address.is_empty() {
                let (block, rest) = samples_address.split_at(std::cmp::min(block_size, samples_address.len()));
                let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
                let (input_address, mut output_address) = process_block.create_and_write_addresses();
                filter.process(&input_address, &mut output_address);

                for channel in process_block.output_block {
                    for i in channel{
                        final_output.push(i);
                    }
                }
                samples_address = rest;
            }
            let actual_vector: Vec<f32> = vec![
                4.27246094e-04,
                1.31958008e-01,
                -1.04217529e-01,
                3.72131348e-01,
                9.37637329e-01,
                1.85363770e-01,
                3.97346497e-01,
                9.25903320e-02,
                2.29892731e-01,
                5.33447266e-02
            ];
            for (i,j) in actual_vector.into_iter().zip(final_output.into_iter()) {
                assert!(is_close(i, j));
            }
            // assert_eq!(actual_vector, final_output);
        }

        /// Must pass test_mono_channel_blocking();
        #[test]
        fn test_mono_process_fir_impulse_for_larger_than_buffer_size_audio() {
            let block_size = 2;
            let channels = 1;
            let mut filter = CombFilter::new(FilterType::FIR, 0.2, 10.0, channels);
            
            let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
            let mut samples_address = &samples[..];

            let mut final_output: Vec<f32> = Vec::new();

            while !samples_address.is_empty() {
                let (block, rest) = samples_address.split_at(std::cmp::min(block_size, samples_address.len()));

                let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);
                let (input_address, mut output_address) = process_block.create_and_write_addresses();
                filter.process(&input_address, &mut output_address);
                for channel in process_block.output_block {
                    for i in channel{
                        final_output.push(i);
                    }
                }

                samples_address = rest;
            }
            let actual_vector: Vec<f32> = vec![
                4.27246094e-04,
                1.31958008e-01,
                -1.04217529e-01,
                3.72131348e-01,
                9.37530518e-01,
                1.52374268e-01,
                4.23400879e-01,
                -4.42504883e-04,
                -4.51660156e-03,
                7.00378418e-03
            ];
            for (i,j) in actual_vector.into_iter().zip(final_output.into_iter()) {
                assert!(is_close(i, j));
            }
            // assert_eq!(actual_vector, final_output);
        }

        /// Must pass `test_stereo_channel_blocking()`;
        #[test]
        fn test_stereo_process_iir_impulse_for_larger_than_buffer_size_audio() {
            let block_size = 2;
            let channels = 2;
            let mut filter = CombFilter::new(FilterType::IIR, 0.2, 5.0, channels);
            
            let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
            let mut samples_address = &samples[..];

            let mut final_output_left: Vec<f32> = Vec::new();
            let mut final_output_right: Vec<f32> = Vec::new();

            while !samples_address.is_empty() {
                let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));

                let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);

                let (input_address, mut output_address) = process_block.create_and_write_addresses();
                filter.process(&input_address, &mut output_address);
                
                for channel in process_block.output_block[0].clone() {
                    final_output_left.push(channel);
                }
                for channel in process_block.output_block[1].clone() {
                    final_output_right.push(channel);
                }

                samples_address = rest;
            }
            let actual_left: Vec<f32> = vec![4.27246094e-04, -1.04217529e-01, 9.37637329e-01, 3.97346497e-01, 2.29892731e-01];
            let actual_right: Vec<f32> = vec![0.13195801, 0.37213135, 0.18536377, 0.09259033, 0.05334473];
            for (i,j) in actual_left.into_iter().zip(final_output_left.into_iter()) {
                assert!(is_close(i, j));
            }
            for (i,j) in actual_right.into_iter().zip(final_output_right.into_iter()) {
                assert!(is_close(i, j));
            }
            // assert_eq!(actual_left, final_output_left);
            // assert_eq!(actual_right, final_output_right);
        }

        /// Must pass `test_stereo_channel_blocking()`;
        #[test]
        fn test_stereo_process_fir_impulse_for_larger_than_buffer_size_audio() {
            
            let block_size = 2;
            let channels = 2;
            let mut filter = CombFilter::new(FilterType::FIR, 0.2, 5.0, channels);
            
            let samples: Vec<i16> = vec![14, 4324, -3422, 10032, 32432, -23, -2342, -3, 1023, 231];
            let mut samples_address = &samples[..];

            let mut final_output_left: Vec<f32> = Vec::new();
            let mut final_output_right: Vec<f32> = Vec::new();

            while !samples_address.is_empty() {
                let (block, rest) = samples_address.split_at(std::cmp::min(block_size*channels, samples_address.len()));

                let mut process_block = ProcessBlocks::new(&block.to_vec(), &channels);

                let (input_address, mut output_address) = process_block.create_and_write_addresses();
                filter.process(&input_address, &mut output_address);
                
                for channel in process_block.output_block[0].clone() {
                    final_output_left.push(channel);
                }
                for channel in process_block.output_block[1].clone() {
                    final_output_right.push(channel);
                }

                samples_address = rest;
            }
            let actual_left: Vec<f32> = vec![4.27246094e-04, -1.04217529e-01,  9.37530518e-01,  4.23400879e-01, -4.51660156e-03];
            let actual_right: Vec<f32> = vec![0.13195801, 0.37213135, 0.15237427, -0.0004425, 0.00700378];
            for (i,j) in actual_left.into_iter().zip(final_output_left.into_iter()) {
                assert!(is_close(i, j));
            }
            for (i,j) in actual_right.into_iter().zip(final_output_right.into_iter()) {
                assert!(is_close(i, j));
            }
            // assert_eq!(actual_left, final_output_left);
            // assert_eq!(actual_right, final_output_right);
        }
    }

    #[test]
    fn test_process_fir_impulse_rand_x_mono() {
        let mut filter = CombFilter::new(FilterType::FIR, 0.5, 10.0, 1);
        filter.set_param(FilterParam::Delay, 0.2).unwrap();
        
        let mut process_block = ProcessBlocks::new(&vec![0; 10], &1);
        process_block.input_block = vec![vec![
            0.35718214,
            0.71631462,
            0.17056465,
            0.00772361,
            0.29818118,
            0.197505,
            0.76061135,
            0.08108575,
            0.64278864,
            0.93702414
        ]];
        process_block.output_block = vec![vec![0.0; 10]];

        let (input_address, mut output_address) = process_block.create_and_write_addresses();

        filter.process(&input_address, &mut output_address);

        // Checked on python implementation
        let actual_value: [f32; 10] = [0.35718214, 0.71631462, 0.34915572, 0.36588091, 0.3834635, 0.2013668 , 0.90970194, 0.17983825, 1.02309432, 0.97756701];
        let final_output = process_block.output_block[0].clone();
        for (a,b) in final_output.into_iter().zip(actual_value){
            assert!(utils::is_close(a, b));
        }
    }

    #[test]
    fn test_process_iir_impulse_rand_x_mono() {
        let mut filter = CombFilter::new(FilterType::IIR, 0.5, 10.0, 1);
        filter.set_param(FilterParam::Delay, 0.2).unwrap();

        let mut process_block = ProcessBlocks::new(&vec![0; 10], &1);
        process_block.input_block = vec![vec![
            0.35718214,
            0.71631462,
            0.17056465,
            0.00772361,
            0.29818118,
            0.197505,
            0.76061135,
            0.08108575,
            0.64278864,
            0.93702414
        ]];
        process_block.output_block = vec![vec![0.0; 10]];

        let (input_address, mut output_address) = process_block.create_and_write_addresses();

        filter.process(&input_address, &mut output_address);

        // Checked on python implementation
        let actual_value: [f32; 10] = [0.35718214, 0.71631462, 0.34915572, 0.36588091, 0.47275904, 0.38044546, 0.99699087, 0.27130848, 1.14128408, 1.07267838];
        let final_output = process_block.output_block[0].clone();
        for (a,b) in final_output.into_iter().zip(actual_value){
            assert!(utils::is_close(a, b));
        }
    }
}