use crate::ring_buffer::RingBuffer;
use std::cmp::max;

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain,
}

pub struct FastConvolver<'a> {
    impulse_response: &'a[f32],
    buffer: Vec<f32>,
    block_size: usize,
    mode: ConvolutionMode,
    input_length: usize,
}

impl<'a> FastConvolver<'a> {
    pub fn new(impulse_response: &'a[f32], mode: ConvolutionMode, block_size: usize) -> Self {
        let len_ir = impulse_response.len() - 1;
        let buffer_size = max(len_ir, block_size);    // buffer_size must be the remaining values length = (block_size + IR_len - 1) - block_size

        Self {
            impulse_response: impulse_response,
            buffer: vec![0.0; buffer_size],
            block_size,
            mode,
            input_length: 0,
        }
    }

    pub fn reset(&mut self) {
        self.buffer = vec![0.0; self.block_size];
    }

    pub fn set_block_size(&mut self, block_size: usize){
        let len_ir = self.impulse_response.len() - 1;
        self.block_size = max(block_size, len_ir);
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        match self.mode {
            ConvolutionMode::TimeDomain => {
                let input_len = input.len();
                self.block_size = input_len;
                for (idx1, &sample) in input.iter().enumerate() {
                    for (idx2, &ir_sample) in self.impulse_response.iter().enumerate() {
                        if idx1+idx2 < self.block_size {
                            output[idx1 + idx2] += sample*ir_sample + self.buffer[idx1 + idx2];
                            self.buffer[idx1 + idx2] = 0.0;
                        } else {
                            self.buffer[idx1 + idx2 - self.block_size] += sample*ir_sample;
                        }
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
    pub fn get_output_tail_size(&self) -> usize {
        self.impulse_response.len() - 1
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        let remaining_samples = self.get_output_tail_size();
        for idx in 0..remaining_samples {
            output[idx] = self.buffer[idx];
        }
        // for i in 0..remaining_samples {
        //     output[i] = (0..self.impulse_response.len()).map(|j| {
        //         if i + j < self.input_length {
        //             self.ring_buffer.get(i + j).unwrap() * self.impulse_response[j]
        //         } else {
        //             0.0
        //         }
        //     }).sum();
        // }
    }
}

// #[cfg(test)]
// mod tests {
//     use rand::Rng;

//     use super::*;


//     #[test]
//     fn test_identity_impulse_response() {
//         let mut rng = rand::thread_rng();
//         let impulse_response: Vec<f32> = (0..51).map(|_| rng.gen::<f32>()).collect();
//         println!("impulse_response : {:?}", impulse_response);
//         let mut input_signal = vec![0.0_f32; 10];
//         input_signal[2] = 1.0;
//         let mut output_signal = vec![0.0_f32; 10];
//         let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
//         convolver.process(&input_signal, &mut output_signal);
//         println!("Output Signal: {:?}", output_signal);
//         let expected_output_at_3 = impulse_response[2]; // As calculated
//         println!("Expected Output at Index 3: {}", expected_output_at_3);
//         println!("Actual Output at Index 3: {}", output_signal[3]);
//         assert!((output_signal[3] - expected_output_at_3).abs() < 1e-5);
//     }

//     #[test]
//     fn test_flush() {
//         let mut rng = rand::thread_rng();
//         let impulse_response: Vec<f32> = (0..51).map(|_| rng.gen::<f32>()).collect();
//         println!("impulse_response : {:?}", impulse_response);
//         let mut input_signal = vec![0.0_f32; 10];
//         input_signal[2] = 1.0;
//         let mut output_signal = vec![0.0_f32; 10];
//         let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
//         convolver.process(&input_signal, &mut output_signal);
//         let tail_size = convolver.get_output_tail_size();
//         let mut reverb_tail = vec![0.0_f32; tail_size];
//         convolver.flush(&mut reverb_tail);
//         println!("Reverb Tail: {:?}", reverb_tail);
//         assert!((reverb_tail[0] - impulse_response[2]).abs() < 1e-5);
//         assert!((reverb_tail[1] - impulse_response[1]).abs() < 1e-5);
//         assert!((reverb_tail[2] - impulse_response[0]).abs() < 1e-5);
//     }

//     #[test]
//     fn test_variable_block_sizes() {
//         let mut input_signal = vec![0.0_f32; 10000];
//         input_signal[3] = 1.0;
//         let mut rng = rand::thread_rng();
//         let impulse_response: Vec<f32> = (0..51).map(|_| rng.gen::<f32>()).collect();
//         let block_sizes = vec![1, 13, 1023, 2048, 1, 17, 5000, 1897];
//         let mut output_signal = vec![0.0_f32; input_signal.len()];
//         let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);

//         for &block_size in &block_sizes {
//             let mut start_index = 0;
//             while start_index < input_signal.len() {
//                 let end_index = std::cmp::min(start_index + block_size, input_signal.len());
//                 let block = &input_signal[start_index..end_index];
//                 let output_block = &mut output_signal[start_index..end_index];
//                 convolver.process(block, output_block);
//                 start_index += block_size;
//             }
//             convolver.reset();
//         }
//         println!("impulse_response : {:?}", impulse_response);
//         println!("Output Signal with variable block sizes: {:?}", output_signal);
//         let expected_output_at_3 = impulse_response[3]; // As calculated
//         println!("Expected Output at Index 3: {}", expected_output_at_3);
//         println!("Actual Output at Index 3: {}", output_signal[3]);
//         assert!((output_signal[3] - expected_output_at_3).abs() < 1e-5);
//     }
// }