use hound::WavWriter;
use rustfft::num_traits::abs;

use crate::fast_convolver::{ConvolutionMode, FastConvolver};
use std::time::Instant;

mod ring_buffer;
mod fast_convolver;
mod utils;

use utils::{i16_to_f32, f32_to_i16, ProcessBlocks};

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output wave filename>", args[0]);
        return;
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let block_size = 512;

    // Ensure the audio is mono
    if spec.channels != 1 {
        eprintln!("Only mono audio files are supported.");
        return;
    }

    // Prepare the convolver
    // let impulse_response = vec![0.1, 0.1, 0.1, 0.9, 0.1, 0.1, 0.1];
    // let impulse_response = Vec::new();
    let mut impulse_reader = hound::WavReader::open(&args[3]).unwrap();
    let impulse_response: Vec<f32> = impulse_reader.samples::<i16>()
        .map(|s| i16_to_f32(s.unwrap()))
        .collect();
    let mut convolver = FastConvolver::new(impulse_response, ConvolutionMode::TimeDomain, block_size);

    // Set up WAV writer with the same specifications as the input
    // let output_path = Path::new(&args[2]);
    let mut writer = WavWriter::create(&args[2], spec).expect("Failed to create WAV writer");

    // Read and process audio data
    let mut samples: Vec<i16> = reader.samples::<i16>()
        .map(|s| s.unwrap())
        .collect();
    let len_samples = samples.len();
    if len_samples%block_size != 0 {
        for _ in 0..(block_size - len_samples%block_size) {
            samples.push(0);
        }
    }
    let mut max_sample_value = 0.0;

    // while let Ok(block) = reader.samples::<i16>().take(block_size).collect::<Result<Vec<_>, _>>() {
    let mut process_block = ProcessBlocks::new(&samples);
    let (input_address, mut output_address) = process_block.get_addresses();

    let now = Instant::now();
    convolver.process(&input_address, &mut output_address);
    println!("Process Time: {}", now.elapsed().as_millis());

    let mut output_samples = process_block.output_block;
    let ir_len = convolver.get_output_tail_size();
    let mut output = vec![0.0; ir_len];

    let now = Instant::now();
    convolver.flush(&mut output);
    println!("Flush Time: {}", now.elapsed().as_millis());
    
    for sample in output.into_iter() {
        output_samples.push(sample);
    }
    for sample in output_samples.iter() {
        if max_sample_value <= abs(*sample) {
            max_sample_value = abs(*sample);
        }
    }
    // }

    // Convert processed samples back to i16 and write them to the WAV file
    for &sample in output_samples.iter() {
        let output_sample = f32_to_i16(sample/max_sample_value);
        writer.write_sample(output_sample).expect("Failed to write sample");
    }

    // // Calculate the size needed for the output tail
    // let tail_size = convolver.get_output_tail_size();

    // // Create a buffer for the tail with the appropriate size
    // let mut reverb_tail = vec![0.0; tail_size];

    // // Perform the flush operation
    // convolver.flush(&mut reverb_tail);

    // // Convert flush samples back to i16 and write them to the WAV file
    // for &sample in reverb_tail.iter() {
    //     let output_sample = f32_to_i16(sample);
    //     writer.write_sample(output_sample).expect("Failed to write tail sample");
    // }

    // writer.finalize().expect("Failed to finalize WAV file");
}
