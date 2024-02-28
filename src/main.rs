use std::{fs::File, io::BufWriter};

use vibrato::Vibrato;
use utils::{Processor, ProcessBlocks};

mod ring_buffer;
mod vibrato;
mod utils;
mod lfo;

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
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let output_file = &args[2];
    if spec.bits_per_sample!=16 {
        eprintln!("Bit depth must be 16 bit! Bit depth of the current song: {}", spec.bits_per_sample);
        return
    }

    let block_size: usize = 1024;

    let mut vibrato_filter = Vibrato::new(spec.sample_rate as f32, 5.0, 0.002, channels);
    let mut writer: hound::WavWriter<BufWriter<File>> = hound::WavWriter::create(output_file, spec).expect("Unable to create file");

    while let Ok(block) = reader.samples::<i16>().take(block_size*channels).collect::<Result<Vec<_>, _>>() {
        let mut process_block = ProcessBlocks::new(&block, &channels);
        let (input_address, mut output_address) = process_block.create_and_write_addresses();
        vibrato_filter.process(&input_address, &mut output_address);
        process_block.write_output_samples(&mut writer).unwrap();
        if block.len() < block_size*channels as usize { break }
    }
}
