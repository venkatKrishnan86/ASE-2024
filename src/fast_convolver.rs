use std::cmp::max;
use rustfft::{FftDirection, Fft, num_complex::Complex, algorithm::Radix4};

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain,
}

pub struct FastConvolver {
    buffer: Vec<f32>,
    impulse_response: Vec<f32>,
    block_size: usize,
    mode: ConvolutionMode,
    fft: Option<Radix4<f32>>,
    ifft: Option<Radix4<f32>>
}

impl FastConvolver {
    pub fn new(impulse_response: Vec<f32>, mode: ConvolutionMode, max_block_size: usize) -> Self {
        let fft: Option<Radix4<f32>>;
        let ifft: Option<Radix4<f32>>;
        match mode {
            ConvolutionMode::TimeDomain => {
                fft = None;
                ifft = None;
            },
            ConvolutionMode::FrequencyDomain => {
                fft = Some(Radix4::new(max_block_size*2, FftDirection::Forward));
                ifft = Some(Radix4::new(max_block_size*2, FftDirection::Inverse));
            }
        } 
        Self {
            buffer: vec![0.0; (impulse_response.len()/max_block_size + 2)*max_block_size],
            impulse_response,
            block_size: max_block_size,
            mode,
            fft,
            ifft
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.impulse_response.clear();
    }

    #[allow(dead_code)]
    pub fn set_block_size(&mut self, block_size: usize){
        let len_ir = self.impulse_response.len() - 1;
        self.block_size = max(block_size, len_ir);
    }

    pub fn add_buffer(&mut self, output: &mut [f32]) {
        let length = self.get_output_tail_size();
        for idx in 0..(length-self.block_size) {
            if idx < self.block_size {
                output[idx] = self.buffer[idx];
            }
            self.buffer[idx] = self.buffer[idx+self.block_size]
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.add_buffer(output);
        let output_len = output.len();
        for (main_idx, input_block) in input.chunks(self.block_size).enumerate() {
            for (ir_idx, ir_block) in self.impulse_response.chunks(self.block_size).enumerate() {
                match self.mode {
                    ConvolutionMode::TimeDomain => FastConvolver::time_convolve(input_block, ir_block, output, &mut self.buffer, (main_idx+ir_idx) * self.block_size, output_len),
                    ConvolutionMode::FrequencyDomain => FastConvolver::frequency_convolve(input_block, ir_block, output, &mut self.buffer, (main_idx+ir_idx) * self.block_size, self.block_size, output_len, self.fft.as_ref().unwrap(), self.ifft.as_ref().unwrap()),
                };
            }
        }   
    }

    fn time_convolve(
        input: &[f32], 
        ir: &[f32], 
        output: &mut [f32], 
        flush_buffer: &mut [f32],
        start_idx: usize,
        output_len: usize
    ){
        for (idx1, &sample) in input.iter().enumerate() {
            for (idx2, &ir_sample) in ir.iter().enumerate() {
                let index = start_idx + idx1 + idx2;
                if index < output_len {
                    output[index] += sample*ir_sample;
                } else {
                    flush_buffer[index - output_len] += sample*ir_sample;
                }
            }
        }
    }

    fn frequency_convolve(
        input: &[f32], 
        ir: &[f32], 
        output: &mut [f32], 
        flush_buffer: &mut [f32],
        start_idx: usize,
        block_size: usize,
        output_len: usize,
        fft: &Radix4<f32>,
        ifft: &Radix4<f32>
    ){
        let mut input_values: Vec<Complex<f32>> = input
            .iter()
            .map(|value| Complex{re: *value, im: 0.0f32})
            .collect();
        let mut ir_values: Vec<Complex<f32>> = ir
            .iter()
            .map(|value| Complex{re: *value, im: 0.0f32})
            .collect();
        for _ in 0..(block_size*2 - input.len()) {
            input_values.push(Complex{re: 0.0f32, im: 0.0f32});
        }
        for _ in 0..(block_size*2 - ir.len()) {
            ir_values.push(Complex{re: 0.0f32, im: 0.0f32});
        }
        fft.process(&mut input_values);
        fft.process(&mut ir_values);
        for (input, ir) in input_values.iter_mut().zip(ir_values.iter()) {
            *input *= *ir
        }
        ifft.process(&mut input_values);
        for (idx, input) in input_values.iter().enumerate() {
            let index = start_idx + idx;
            if index < output_len {
                output[index] += input.re/(block_size as f32 * 2.0);
            } else {
                flush_buffer[index - output_len] += input.re/(block_size as f32 * 2.0);
            }
        }
    }

    pub fn get_output_tail_size(&self) -> usize {
        self.impulse_response.len()/self.block_size + 2*self.block_size
    }

    pub fn flush(&mut self, output: &mut Vec<f32>) {
        *output = self.buffer.clone();
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use super::*;

    #[test]
    fn test_identity_impulse_response() {
        let block_size = 16;
        let mut rng = rand::thread_rng();
        let mut impulse_response = vec![0.0; 50];
        impulse_response[0] = 1.0;
        println!("impulse_response : {:?}", impulse_response);
        let input_signal: Vec<f32> = (0..block_size).map(|_| rng.gen::<f32>()).collect();
        let mut output_signal: Vec<f32> = vec![0.0_f32; block_size];
        let mut convolver = FastConvolver::new(impulse_response, ConvolutionMode::TimeDomain, block_size);
        convolver.process(&input_signal, &mut output_signal);
        for (in_val, out_val) in input_signal.iter().zip(output_signal.iter()){
            assert!((out_val - in_val).abs() < 1e-5);
        }
    }

    #[test]
    fn test_identity_impulse_response_frequency() {
        let block_size = 16;
        let mut rng = rand::thread_rng();
        let mut impulse_response = vec![0.0; 50];
        impulse_response[0] = 1.0;
        let input_signal: Vec<f32> = (0..block_size).map(|_| rng.gen::<f32>()).collect();
        let mut output_signal: Vec<f32> = vec![0.0_f32; block_size];
        let mut convolver = FastConvolver::new(impulse_response, ConvolutionMode::FrequencyDomain, block_size);
        convolver.process(&input_signal, &mut output_signal);
        for (in_val, out_val) in input_signal.iter().zip(output_signal.iter()){
            assert!((out_val - in_val).abs() < 1e-5);
        }
    }

    fn test_flush(block_size: usize, mode: ConvolutionMode, ir_length: usize) {
        // let block_size = 16;
        let mut rng = rand::thread_rng();
        let mut impulse_response = vec![0.0; ir_length];
        impulse_response[0] = 1.0;

        let gain = rng.gen::<f32>();

        impulse_response[block_size] = gain; // Will delay the whole input by 10 samples with a random gain
        let input_signal: Vec<f32> = (0..block_size).map(|_| rng.gen::<f32>()).collect();
        let mut output_signal: Vec<f32> = vec![0.0_f32; block_size];
        let mut convolver = FastConvolver::new(impulse_response, mode, block_size);
        convolver.process(&input_signal, &mut output_signal);
        for (in_val, out_val) in input_signal.iter().zip(output_signal.iter()){
            assert!((out_val - in_val).abs() < 1e-5);
        }

        // The reverb tail is checking if the delay is done correctly
        let tail_len = convolver.get_output_tail_size();
        let mut reverb_tail: Vec<f32> = vec![0.0_f32; tail_len];
        convolver.flush(&mut reverb_tail);
        for i in 0..block_size{
            assert!((reverb_tail[i] - input_signal[i]*gain).abs() < 1e-5);   // First 10 tail values must be the delayed input signal times the random gain
        }
        for i in block_size..tail_len{
            assert!(reverb_tail[i].abs() < 1e-5);                            // Rest of the values need to be 0.0
        }
    }

    #[test]
    fn test_flush_time() {
        test_flush(16, ConvolutionMode::TimeDomain, 50)
    }

    #[test]
    fn test_flush_frequency() {
        test_flush(16, ConvolutionMode::FrequencyDomain, 50)
    }

    #[test]
    fn test_trial() {
        let block_size = 16;
        let mut impulse_response = vec![0.0; 130];
        impulse_response[0] = 1.0;
        let mut rng = rand::thread_rng();
        let input_signal: Vec<f32> = (0..block_size*4).map(|_| rng.gen::<f32>()).collect();
        let mut output_signal: Vec<f32> = vec![0.0; block_size*4];
        let mut convolver = FastConvolver::new(impulse_response, ConvolutionMode::TimeDomain, block_size);
        convolver.process(&input_signal, &mut output_signal);
        for i in 0..block_size*2{
            println!("{} {}", output_signal[i], input_signal[i]);
            assert!((output_signal[i] - input_signal[i]).abs() < 1e-5);   // First 10 tail values must be the delayed input signal times the random gain
        }
    }

    #[test]
    fn test_variable_block_sizes_time() {
        let block_sizes = vec![1, 13, 1023, 2048, 1, 17, 5000, 1897];
        for block_size in block_sizes {
            test_flush(block_size, ConvolutionMode::TimeDomain, 6000);
        }
    }

    #[test]
    fn test_variable_block_sizes_freq() {
        let base: usize = 2;
        let block_sizes = (0..12)
            .into_iter()
            .map(|value| base.pow(value))
            .collect::<Vec<usize>>();
        // block_sizes = [1,2,4,8...,2048]
        for block_size in block_sizes {
            test_flush(block_size, ConvolutionMode::FrequencyDomain, 5000);
        }
    }
}