pub struct ProcessBlocks {
    pub input_block: Vec<Vec<f32>>,
    pub output_block: Vec<Vec<f32>>
}

impl ProcessBlocks {
    pub fn new(audio: &Vec<i16>, channels: &usize) -> Self {
        let length = audio.len();
        let mut block = Self {
            input_block: vec![vec![0.0; length/channels]; *channels],
            output_block: vec![vec![0.0; length/channels]; *channels]
        };
        block.convert_i16_samples_to_f32(audio, channels);
        block
    }

    fn convert_i16_samples_to_f32(&mut self, audio: &Vec<i16>, channels: &usize) {
        for (i, &sample) in audio.iter().enumerate() {
            let sample = i16_to_f32(sample);
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

pub fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}

pub fn f32_to_i16(value: f32) -> i16 {
    assert!(value>=-1.0 && value<1.0);
    (value*(-(i16::MIN as f32))) as i16
}

pub fn i16_to_f32(value: i16) -> f32 {
    value as f32 / (1 << 15) as f32
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
            for output_value in output_channel.iter() {
                assert!(is_close(0.0, *output_value));
            }
        }
        assert_eq!(3, counter);
    }

    #[test]
    #[should_panic]
    fn test_is_close_function_and_max_sample() {
        let block = ProcessBlocks::new(&vec![i16::MAX], &1);
        assert!(is_close(1.0, block.input_block[0][0]));
    }

    #[test]
    fn converting_samples_test_zero() {
        let block = ProcessBlocks::new(&vec![0], &1);
        assert!(is_close(0.0, block.input_block[0][0]));
    }

    #[test]
    fn converting_samples_test_min() {
        let block = ProcessBlocks::new(&vec![i16::MIN, 0], &1);
        assert!(is_close(-1.0, block.input_block[0][0]));
        assert!(is_close(0.0, block.input_block[0][1]));
    }

    #[test]
    fn converting_samples_test_max() {
        let block = ProcessBlocks::new(&vec![i16::MAX, 0], &1);
        assert!(is_close(i16_to_f32(i16::MAX), block.input_block[0][0]));
        assert!(is_close(0.0, block.input_block[0][1]));
    }

    #[test]
    fn creating_and_writing_addresses_test_correct_pointer() {
        let mut block = ProcessBlocks::new(&vec![i16::MIN, 0], &1);
        let (a, b) = &block.create_and_write_addresses();
        for (value_a, value_b) in a.into_iter().zip(b.into_iter()) {
            assert_eq!(*value_a, [-1.0, 0.0]);
            assert_eq!(*value_b, [0.0, 0.0]);
        }
    }

    #[test]
    fn creating_and_writing_addresses_test_mutability_check() {
        let mut block = ProcessBlocks::new(&vec![i16::MIN, 0, 12, 4932,-2345], &1);
        let (_, b) = block.create_and_write_addresses();
        for value_b in b.into_iter() {
            value_b[0] = 1.0;
            value_b[2] = 0.2
        }
    }

    #[test]
    fn test_transpose() {
        let a: Vec<Vec<i32>> = vec![
            vec![1,2,3],
            vec![4,5,6]
        ];
        assert_eq!(transpose(a), vec![
            vec![1,4],
            vec![2,5],
            vec![3,6],
        ]);
    }

    #[test]
    fn test_f32_to_i16_1() {
        assert_eq!(f32_to_i16(-1.0), i16::MIN);
    }

    #[test]
    #[should_panic]
    fn test_f32_to_i16_at_one() {
        f32_to_i16(1.0);
    }

    #[test]
    #[should_panic]
    fn test_f32_to_i16_for_less_than_minus_one() {
        f32_to_i16(-1.01);
    }
}