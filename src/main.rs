use std::{fs::File, io::Write};

use hound::WavWriter;
use utils::ProcessBlocks;
use comb_filter::{CombFilter, FilterType};

mod comb_filter;
mod utils;

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
    let channels: usize = spec.channels as usize;
    // const channels: u16 = 2;

    // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
    //       Use the following block size:
    let block_size: usize = 1024*64;

    // Read audio data and write it to the output text file (one column per channel)
    // let out: File = File::create(&args[2]).expect("Unable to create file");

    let mut comb_filter_1 = CombFilter::new(FilterType::FIR, 0.1, 44100.0, channels);
    let mut writer = WavWriter::create(&args[2], spec).expect("Unable to create file");

    while let Ok(block) = reader.samples::<i16>().take(block_size*channels as usize).collect::<Result<Vec<_>, _>>() {
        let mut process_block = ProcessBlocks::new(&block, &channels);
        let (input_address, mut output_address) = process_block.create_and_write_addresses();

        comb_filter_1.process(&input_address, &mut output_address);

        let output_block: Vec<Vec<f32>> = utils::transpose(process_block.output_block);

        for output in output_block.into_iter() {
            for value in output {
                writer.write_sample(utils::f32_to_i16(value)).unwrap();
            }
        }

        if block.len() < block_size*channels as usize { break }
    }
}
