use crate::ring_buffer::RingBuffer;

#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    TimeDomain,
    FrequencyDomain { block_size: usize },
}

pub struct FastConvolver {
    impulse_response: Vec<f32>,
    ring_buffer: RingBuffer<f32>,
    mode: ConvolutionMode,
    input_length: usize,
}

impl FastConvolver {
    pub fn new(impulse_response: &[f32], mode: ConvolutionMode) -> Self {
        let buffer_size = match mode {
            ConvolutionMode::TimeDomain => impulse_response.len(),
            ConvolutionMode::FrequencyDomain { block_size } => block_size + impulse_response.len() - 1,
        };

        FastConvolver {
            impulse_response: impulse_response.to_vec(),
            ring_buffer: RingBuffer::new(buffer_size),
            mode,
            input_length: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ring_buffer.reset();
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        match self.mode {
            ConvolutionMode::TimeDomain => {
                for (i, &sample) in input.iter().enumerate() {
                    self.ring_buffer.push(sample);
                    output[i] = (0..self.impulse_response.len()).map(|j| {
                        self.ring_buffer.get(j) * self.impulse_response[j]
                    }).sum();
                }
                self.input_length = input.len();
            }
            _ => unimplemented!(),
        }
    }
    pub fn get_output_tail_size(&self) -> usize {
        self.input_length % self.impulse_response.len()
    }

    pub fn flush(&mut self, output: &mut [f32]) {
        let remaining_samples = self.get_output_tail_size();
        for i in 0..remaining_samples {
            output[i] = (0..self.impulse_response.len()).map(|j| {
                if i + j < self.input_length {
                    self.ring_buffer.get(i + j) * self.impulse_response[j]
                } else {
                    0.0
                }
            }).sum();
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;


    #[test]
    fn test_identity_impulse_response() {
        let mut rng = rand::thread_rng();
        let impulse_response: Vec<f32> = (0..51).map(|_| rng.gen::<f32>()).collect();
        println!("impulse_response : {:?}", impulse_response);
        let mut input_signal = vec![0.0_f32; 10];
        input_signal[2] = 1.0;
        let mut output_signal = vec![0.0_f32; 10];
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        convolver.process(&input_signal, &mut output_signal);
        println!("Output Signal: {:?}", output_signal);
        let expected_output_at_3 = impulse_response[2]; // As calculated
        println!("Expected Output at Index 3: {}", expected_output_at_3);
        println!("Actual Output at Index 3: {}", output_signal[3]);
        assert!((output_signal[3] - expected_output_at_3).abs() < 1e-5);
    }

    #[test]
    fn test_flush() {
        let mut rng = rand::thread_rng();
        let impulse_response: Vec<f32> = (0..51).map(|_| rng.gen::<f32>()).collect();
        println!("impulse_response : {:?}", impulse_response);
        let mut input_signal = vec![0.0_f32; 10];
        input_signal[2] = 1.0;
        let mut output_signal = vec![0.0_f32; 10];
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);
        convolver.process(&input_signal, &mut output_signal);
        let tail_size = convolver.get_output_tail_size();
        let mut reverb_tail = vec![0.0_f32; tail_size];
        convolver.flush(&mut reverb_tail);
        println!("Reverb Tail: {:?}", reverb_tail);
        assert!((reverb_tail[0] - impulse_response[2]).abs() < 1e-5);
        assert!((reverb_tail[1] - impulse_response[1]).abs() < 1e-5);
        assert!((reverb_tail[2] - impulse_response[0]).abs() < 1e-5);
    }

    #[test]
    fn test_variable_block_sizes() {
        let mut input_signal = vec![0.0_f32; 10000];
        input_signal[3] = 1.0;
        let mut rng = rand::thread_rng();
        let impulse_response: Vec<f32> = (0..51).map(|_| rng.gen::<f32>()).collect();
        let block_sizes = vec![1, 13, 1023, 2048, 1, 17, 5000, 1897];
        let mut output_signal = vec![0.0_f32; input_signal.len()];
        let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);

        for &block_size in &block_sizes {
            let mut start_index = 0;
            while start_index < input_signal.len() {
                let end_index = std::cmp::min(start_index + block_size, input_signal.len());
                let block = &input_signal[start_index..end_index];
                let output_block = &mut output_signal[start_index..end_index];
                convolver.process(block, output_block);
                start_index += block_size;
            }
            convolver.reset();
        }
        println!("impulse_response : {:?}", impulse_response);
        println!("Output Signal with variable block sizes: {:?}", output_signal);
        let expected_output_at_3 = impulse_response[3]; // As calculated
        println!("Expected Output at Index 3: {}", expected_output_at_3);
        println!("Actual Output at Index 3: {}", output_signal[3]);
        assert!((output_signal[3] - expected_output_at_3).abs() < 1e-5);
    }
}