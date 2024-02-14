use std::{fs::File, io::Write};

use utils::ProcessBlocks;

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
    let block_size: usize = 1024;

    // Read audio data and write it to the output text file (one column per channel)
    let mut out: File = File::create(&args[2]).expect("Unable to create file");

    let mut comb_filter_1 = comb_filter::CombFilter::new(comb_filter::FilterType::FIR, 0.1, 44100.0, channels as usize);
    let mut channel_output_values: Vec<String> = Vec::new();
    for _ in 0..channels {
        channel_output_values.push(String::from(""));
    }

    while let Ok(block) = reader.samples::<i16>().take(block_size*channels as usize).collect::<Result<Vec<_>, _>>() {
        let mut process_block = ProcessBlocks::new(&block, &channels);
        process_block.convert_i16_samples_to_f32(&block, &channels);
        let (input_address, mut output_address) = process_block.create_and_write_addresses();

        comb_filter_1.process(&input_address, &mut output_address);

        for (i, output_block) in process_block.output_block.into_iter().enumerate() {
            for value in output_block {
                channel_output_values[i].push_str(&format!("{}\n", value));
            }
        }

        if block.len() < block_size*channels as usize { break }
    }
    for channel_sample in channel_output_values {
        write!(out, "{}", channel_sample).unwrap();
    }
}
