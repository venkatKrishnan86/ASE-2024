use std::io::Write;

use hound::WavWriter;

use crate::fast_convolver::{ConvolutionMode, FastConvolver};

mod ring_buffer;
mod fast_convolver;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output text filename>", args[0]);
        return;
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();

    // Ensure the audio is mono
    if spec.channels != 1 {
        eprintln!("Only mono audio files are supported.");
        return;
    }

    // Prepare the convolver
    let impulse_response = vec![0.1, 0.1, 0.1, 0.9, 0.1, 0.1, 0.1];
    let mut convolver = FastConvolver::new(&impulse_response, ConvolutionMode::TimeDomain);

    // Set up WAV writer with the same specifications as the input
    // let output_path = Path::new(&args[2]);
    let mut writer = WavWriter::create(&args[2], spec).expect("Failed to create WAV writer");

    // Read and process audio data
    let samples: Vec<f32> = reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / (std::i16::MAX as f32 + 1.0))
        .collect();

    let mut output_samples = vec![0.0; samples.len()];
    convolver.process(&samples, &mut output_samples);

    // Convert processed samples back to i16 and write them to the WAV file
    for &sample in output_samples.iter() {
        let output_sample = (sample * (std::i16::MAX as f32 + 1.0)) as i16;
        writer.write_sample(output_sample).expect("Failed to write sample");
    }

    // Calculate the size needed for the output tail
    let tail_size = convolver.get_output_tail_size();

    // Create a buffer for the tail with the appropriate size
    let mut reverb_tail = vec![0.0; tail_size];

    // Perform the flush operation
    convolver.flush(&mut reverb_tail);

    // Convert flush samples back to i16 and write them to the WAV file
    for &sample in reverb_tail.iter() {
        let output_sample = (sample * (std::i16::MAX as f32 + 1.0)) as i16;
        writer.write_sample(output_sample).expect("Failed to write tail sample");
    }

    writer.finalize().expect("Failed to finalize WAV file");
}
