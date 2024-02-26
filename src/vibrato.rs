use crate::ring_buffer::RingBuffer;

enum FilterParam {
    ModFreq,
    Width
}

trait Processor {
    type Item;

    fn process(&self);
    fn get_param(&self, param: FilterParam) -> Self::Item;
    fn set_param(&mut self, param: FilterParam, value: Self::Item) -> Result<(), String>;
}

struct Vibrato<T: Copy + Default> {
    sample_rate_hz: usize,
    mod_freq: T,
    width: T,
    delay_line: RingBuffer<T>
}

impl<T> Vibrato<T>
where T: Copy + Default
{
    fn new(sample_rate_hz: usize, mod_freq: T, width: T, buffer_size: usize) -> Self {
        Self {
            sample_rate_hz,
            mod_freq,
            width,
            delay_line: RingBuffer::new(buffer_size)
        }
    }
}

impl Processor for Vibrato<f32> 
{
    type Item = f32;

    fn get_param(&self, param: FilterParam) -> Self::Item {
        match param {
            FilterParam::ModFreq => self.mod_freq,
            FilterParam::Width => self.width
        }
    }

    fn process(&self) {
        
    }

    fn set_param(&mut self, param: FilterParam, value: Self::Item) -> Result<(), String> {
        if value <= 0.0 {
            return Err("Value must be positive!".to_owned())
        }
        match param {
            FilterParam::ModFreq => self.mod_freq = value,
            FilterParam::Width => self.width = value
        }
        Ok(())
    }
}

impl Processor for Vibrato<i16> 
{
    type Item = i16;

    fn get_param(&self, param: FilterParam) -> Self::Item {
        match param {
            FilterParam::ModFreq => self.mod_freq,
            FilterParam::Width => self.width
        }
    }

    fn process(&self) {
        
    }

    fn set_param(&mut self, param: FilterParam, value: Self::Item) -> Result<(), String> {
        if value <= 0 {
            return Err("Value must be positive!".to_owned())
        }
        match param {
            FilterParam::ModFreq => self.mod_freq = value,
            FilterParam::Width => self.width = value
        }
        Ok(())
    }
}