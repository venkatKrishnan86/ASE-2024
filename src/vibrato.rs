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

    /// Resets the -
    /// 1. Delay Line
    /// 2. LFO wavetable
    /// back to zeros, and restarts the sample indices
    fn reset(&mut self) {
        for channel in 0..self.num_channels{
            self.delay_line[channel].reset();
            self.lfo[channel].reset();
            for _ in 0..(2 + 3*self.width) {
                self.delay_line[channel].push(Self::Item::default());
            }
        }
    }

    /// Get the value of the requested parameter
    ///
    /// ## Arguments
    ///
    /// `param: FilterParam`: An enum value describing which parameter needs to be obtained
    ///
    /// ## Returns
    /// 
    /// `Self::Item`: Returns the (f32) value of the filter parameter
    fn get_param(&self, param: FilterParam) -> Self::Item {
        match param {
            FilterParam::ModFreq => {
                (1.0/self.lfo[0].size() as f32) * self.sample_rate_hz
            },
            FilterParam::Width => self.width as f32/self.sample_rate_hz
        }
    }

    /// Process the audio in form of blocks
    ///
    /// ## Arguments
    ///
    /// `input: &[&[Self::Item]]`: Input reference of shape (number of channels, length of audio array)
    /// `output: &mut[&mut[Self::Item]]`: Output reference of shape (number of channels, length of audio array)
    ///
    /// The output audio is updated post this function
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

    /// Sets the parameter value of user's choice
    ///
    /// ## Arguments
    ///
    /// `param: FilterParam`: An enum value describing which parameter needs to be modified
    /// `value: Self::Item`: The value which will be updated (here, the `Self::Item` is `f32`)
    ///
    /// ## Returns
    /// 
    /// `Result<(), String>`: `Ok(())` if the value inputted is >= 0.0 else will return an error with an appropriate error message
    fn set_param(&mut self, param: FilterParam, value: Self::Item) -> Result<(), String> {
        if value <= 0.0 {
            return Err(format!("Value inputted {value} must be positive!").to_owned())
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

        #[test]
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