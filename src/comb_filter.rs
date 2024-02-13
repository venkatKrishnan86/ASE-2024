use ringbuffer::{AllocRingBuffer, RingBuffer};

pub struct CombFilter {
    // TODO: your code here
    filter_type: FilterType,
    delay: f32,
    delayed_signal_amp_factor: f32,
    sample_rate_hz: f32,
    num_channels: usize,
    delay_line: AllocRingBuffer<f32>
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
        Self {
            filter_type: filter_type,
            delay: max_delay_secs,
            delayed_signal_amp_factor: 0.5,
            sample_rate_hz: sample_rate_hz,
            num_channels: num_channels,
            delay_line: AllocRingBuffer::with_capacity((max_delay_secs * sample_rate_hz) as usize)
        }
    }

    pub fn reset(&mut self) {
        self.delay_line.clear();
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        match self.filter_type {
            FilterType::FIR => {
                for (input_channel, output_channel) in input.iter().zip(output.iter_mut()) {
                    for (input_sample, output_sample) in input_channel.iter().zip(output_channel.iter_mut()) {
                        *output_sample = input_sample + self.delayed_signal_amp_factor * self.delay_line.peek().unwrap_or(&0.0);
                        self.delay_line.push(*input_sample);
                    }
                }
            },
            FilterType::IIR => {
                for (input_block, output_block) in input.iter().zip(output.iter_mut()) {
                    for (input_sample, output_sample) in input_block.iter().zip(output_block.iter_mut()) {
                        *output_sample = input_sample + self.delayed_signal_amp_factor * self.delay_line.peek().unwrap();
                        self.delay_line.push(*output_sample);
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
                if value > self.delay_line.capacity() as f32/self.sample_rate_hz || value < 0.0 {
                    return Err(Error::InvalidValue { param: param, value: value });
                }
                self.delay = value
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

#[cfg(test)]
mod tests {
    use super::*;

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
}