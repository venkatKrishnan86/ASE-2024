pub struct ProcessBlocks {
    pub input_block: Vec<Vec<f32>>,
    pub output_block: Vec<Vec<f32>>
}

impl ProcessBlocks {
    pub fn new(audio: &Vec<i16>, channels: &usize) -> Self {
        let length = audio.len();
        Self {
            input_block: vec![vec![0.0; length/channels]; *channels],
            output_block: vec![vec![0.0; length/channels]; *channels]
        }
    }

    pub fn convert_i16_samples_to_f32(&mut self, audio: &Vec<i16>, channels: &usize) {
        for (i, &sample) in audio.iter().enumerate() {
            let sample = sample as f32 / (1 << 15) as f32;
            let channel = i % channels;
            let index = i / channels;
            self.input_block[channel][index] = sample;
        }
    }

    pub fn create_and_write_addresses(&mut self) -> (Vec<&[f32]>, Vec<&mut [f32]>) {
        let mut input_immut_block: Vec<&[f32]> = Vec::new();
        let mut output_mut_block: Vec<&mut [f32]> = Vec::new();
        for (input_blocks, output_blocks) in self.input_block.iter().zip(self.output_block.iter_mut()) {
            input_immut_block.push(input_blocks);
            output_mut_block.push(output_blocks);
        }
        (input_immut_block, output_mut_block)
    }
}

pub fn is_close(a: f32, b: f32) -> bool {
    (a-b).abs() < f32::EPSILON
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn process_block_creation_test() {
        let block = ProcessBlocks::new(&vec![1,4,-3,6,-2,3], &3);
        let mut counter: usize = 0;
        for (input_channel, output_channel) in block.input_block.iter().zip(block.output_block.iter()){
            counter+=1;
            assert_eq!(2, input_channel.len());
            assert_eq!(2, output_channel.len());
            for (input_value, output_value) in input_channel.iter().zip(output_channel.iter()) {
                assert!(is_close(0.0, *input_value));
                assert!(is_close(0.0, *output_value));
            }
        }
        assert_eq!(3, counter);
    }

    #[test]
    fn converting_samples_test_zero() {
        let mut block = ProcessBlocks::new(&vec![0], &1);
        block.convert_i16_samples_to_f32(&vec![0], &1);
        assert!(is_close(0.0, block.input_block[0][0]));
    }

    #[test]
    fn converting_samples_test_min() {
        let mut block = ProcessBlocks::new(&vec![i16::MIN, 0], &1);
        block.convert_i16_samples_to_f32(&vec![i16::MIN], &1);
        assert!(is_close(-1.0, block.input_block[0][0]));
        assert!(is_close(0.0, block.input_block[0][1]));
    }

    #[test]
    fn creating_and_writing_addresses_test_correct_pointer() {
        let mut block = ProcessBlocks::new(&vec![i16::MIN, 0], &1);
        block.convert_i16_samples_to_f32(&vec![i16::MIN, 0], &1);
        let (a, b) = &block.create_and_write_addresses();
        for (value_a, value_b) in a.into_iter().zip(b.into_iter()) {
            assert_eq!(*value_a, [-1.0, 0.0]);
            assert_eq!(*value_b, [0.0, 0.0]);
        }
    }

    #[test]
    fn creating_and_writing_addresses_test_mutability_check() {
        let mut block = ProcessBlocks::new(&vec![i16::MIN, 0, 12, 4932,-2345], &1);
        block.convert_i16_samples_to_f32(&vec![i16::MIN, 0, 12, 4932,-2345], &1);
        let (_, b) = block.create_and_write_addresses();
        for value_b in b.into_iter() {
            value_b[0] = 1.0;
            value_b[2] = 0.2
        }
    }
}