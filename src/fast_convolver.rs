use std::cmp::{max,min};
use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain,
}

pub struct FastConvolver {
    impulse_response: Vec<f32>,
    buffer: RingBuffer<RingBuffer<Vec<f32>>>,
    buffer_size: usize,
    block_size: usize,
    mode: ConvolutionMode
}

impl FastConvolver {
    pub fn new(impulse_response: Vec<f32>, mode: ConvolutionMode, max_block_size: usize) -> Self {
        let mut ir = impulse_response;
        let len_ir = ir.len();
        let pad_value = len_ir%max_block_size;
        if pad_value != 0 {
            for _ in 0..(max_block_size-pad_value) {
                ir.push(0.0);
            }
        }
        let buffer_size = len_ir/max_block_size + 1;

        // To match the broken pieces  (4+1 = 5 as in DAFX example) we will use buffer_size+1
        Self {
            impulse_response: ir,
            buffer: RingBuffer::new(buffer_size+1, RingBuffer::new(buffer_size*buffer_size, Vec::new())),
            buffer_size: buffer_size,
            block_size: max_block_size,
            mode
        }
    }

    pub fn reset(&mut self) {
        self.buffer.reset();
        for _ in 0..(self.buffer_size+1) {
            let new_ring_buffer: RingBuffer<Vec<f32>> = RingBuffer::new(self.buffer_size*self.buffer_size, Vec::new());
            self.buffer.push(new_ring_buffer);
        }
    }

    pub fn set_block_size(&mut self, block_size: usize){
        let len_ir = self.impulse_response.len() - 1;
        self.block_size = max(block_size, len_ir);
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.reset();
        for (main_idx, (input_block, output_block)) in input.chunks(self.block_size).zip(output.chunks_mut(self.block_size)).enumerate() {
            for (ir_idx, ir_block) in self.impulse_response.chunks(self.block_size).enumerate() {
                println!("{main_idx}: {ir_idx} Iteration running");
                for offset in 0..(self.buffer_size+1) {
                    let new_ring_buffer = self.buffer.get_mut(offset).unwrap();
                    if new_ring_buffer.len() == new_ring_buffer.capacity() {
                        new_ring_buffer.pop();
                    }
                    new_ring_buffer.push(vec![0.0; self.block_size]);
                }
                let output_loc = self.buffer.get_mut(ir_idx).unwrap();
                let output_temp = output_loc.get_mut(output_loc.get_write_index()).unwrap();
                let output_flush = match self.mode {
                    ConvolutionMode::TimeDomain => FastConvolver::time_convolve(input_block, ir_block, output_temp, self.block_size),
                    ConvolutionMode::FrequencyDomain => unimplemented!()
                };
                let output_loc = self.buffer.get_mut(ir_idx+1).unwrap();
                let output_temp = output_loc.get_mut(output_loc.get_write_index()).unwrap();
                *output_temp = output_flush;
            }
            let curr_buffer = self.buffer.pop().unwrap();
            for results in curr_buffer.into_iter() {
                for idx in 0..results.len() {
                    output_block[idx] += results[idx]
                }
            }
            self.buffer.push(RingBuffer::new(self.buffer_size*self.buffer_size, Vec::new()));
        }   
    }

    fn time_convolve(input: &[f32], ir: &[f32], output: &mut [f32], block_size: usize) -> Vec<f32>{
        let mut output_flush = vec![0.0; block_size];
        for (idx1, &sample) in input.iter().enumerate() {
            for (idx2, &ir_sample) in ir.iter().enumerate() {
                if idx1+idx2 < block_size {
                    output[idx1 + idx2] += sample*ir_sample;
                } else {
                    output_flush[idx1 + idx2 - block_size] = sample*ir_sample;
                }
            }
        }
        return output_flush
    }

    pub fn get_output_tail_size(&self) -> usize {
        self.impulse_response.len() - 1
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        let remaining_samples = self.get_output_tail_size();
        for idx in 0..remaining_samples {
            // output[idx] = self.buffer[idx];
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

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;


    #[test]
    fn test_identity_impulse_response() {
        let mut rng = rand::thread_rng();
        let mut impulse_response = vec![0.0; 50];
        impulse_response[0] = 1.0;
        println!("impulse_response : {:?}", impulse_response);
        let input_signal: Vec<f32> = (0..10).map(|_| rng.gen::<f32>()).collect();
        let mut output_signal: Vec<f32> = vec![0.0_f32; 10];
        let mut convolver = FastConvolver::new(impulse_response, ConvolutionMode::TimeDomain, 10);
        convolver.process(&input_signal, &mut output_signal);
        for (in_val, out_val) in input_signal.iter().zip(output_signal.iter()){
            assert!((out_val - in_val).abs() < 1e-5);
        }
    }

    #[test]
    fn test_flush() {
        let mut rng = rand::thread_rng();
        let mut impulse_response = vec![0.0; 50];
        impulse_response[0] = 1.0;

        let gain = rng.gen::<f32>();

        impulse_response[10] = gain; // Will delay the whole input by 10 samples with a random gain
        let input_signal: Vec<f32> = (0..10).map(|_| rng.gen::<f32>()).collect();
        let mut output_signal: Vec<f32> = vec![0.0_f32; 10];
        let mut convolver = FastConvolver::new(impulse_response, ConvolutionMode::TimeDomain, 10);
        convolver.process(&input_signal, &mut output_signal);
        // for (in_val, out_val) in input_signal.iter().zip(output_signal.iter()){
        //     assert!((out_val - in_val).abs() < 1e-5);
        // }

        // The reverb tail is checking if the delay is done correctly
        let tail_len = convolver.get_output_tail_size();
        let mut reverb_tail: Vec<f32> = vec![0.0_f32; tail_len];
        convolver.flush(&mut reverb_tail);
        for i in 0..10{
            assert!((reverb_tail[i] - input_signal[i]*gain).abs() < 1e-5);   // First 10 tail values must be the delayed input signal times the random gain
        }
        for i in 10..tail_len{
            assert!(reverb_tail[i].abs() < 1e-5);                            // Rest of the values need to be 0.0
        }
    }

//     #[test]
//     fn test_variable_block_sizes() {
//         let mut input_signal = vec![0.0_f32; 10000];
//         input_signal[3] = 1.0;
//         let mut rng = rand::thread_rng();
//         let impulse_response: Vec<f32> = (0..51).map(|_| rng.gen::<f32>()).collect();
//         let block_sizes = vec![1, 13, 1023, 2048, 1, 17, 5000, 1897];
//         let mut output_signal = vec![0.0_f32; input_signal.len()];
//         let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain, 2);

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
}