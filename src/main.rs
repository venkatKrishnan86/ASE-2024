use std::{fs::File, io::BufWriter};

mod ring_buffer;
mod fast_convolver;
mod utils;

use utils::ProcessBlocks;
use fast_convolver::{ConvolutionMode, FastConvolver};

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <input wave filename> <output wave filename> <IR wave filename>", args[0]);
        return
    }

    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let mut impulse_reader = hound::WavReader::open(&args[3]).unwrap();
    let spec = reader.spec();
    let output_file = &args[2];
    if spec.bits_per_sample!=16 {
        eprintln!("Bit depth must be 16 bit! Bit depth of the current song: {}", spec.bits_per_sample);
        return
    }
    if impulse_reader.spec().bits_per_sample!=16 {
        eprintln!("Bit depth must be 16 bit! Bit depth of the current song: {}", impulse_reader.spec().bits_per_sample);
        return
    }

    let block_size: usize = 1024;
    let mut impulse = Vec::new();
    let mut audio = Vec::new();

    while let Ok(block) = impulse_reader.samples::<i16>().take(block_size).collect::<Result<Vec<_>, _>>(){
        for i in block.iter(){
            if i%2 == 0{
                continue;
            }
            impulse.push(utils::i16_to_f32(*i));
        }
        if block.len() < block_size as usize { break }
    }
    while let Ok(block) = reader.samples::<i16>().take(block_size).collect::<Result<Vec<_>, _>>(){
        for i in block.iter(){
            if i%2 == 0{
                continue;
            }
            audio.push(*i);
        }
        if block.len() < block_size as usize { break }
    }
    let mode = ConvolutionMode::TimeDomain;

    let mut convolver = FastConvolver::new(&impulse, mode);
    let mut writer: hound::WavWriter<BufWriter<File>> = hound::WavWriter::create(output_file, spec).expect("Unable to create file");

    match mode {
        ConvolutionMode::TimeDomain => {
            let mut process_block = ProcessBlocks::new(&audio, &impulse);
            let (input_address, output_address) = process_block.create_and_write_addresses();
            dbg!(input_address);
            convolver.process(input_address, output_address);
            dbg!(output_address);
            process_block.write_output_samples(&mut writer).unwrap();
        },
        ConvolutionMode::FrequencyDomain { block_size } => {

        }
    }

    // while let Ok(block) = reader.samples::<i16>().take(block_size).collect::<Result<Vec<_>, _>>() {
    //     let mut process_block = ProcessBlocks::new(&block);
    //     let (input_address, output_address) = process_block.create_and_write_addresses();
    //     convolver.process(input_address, output_address);
    //     process_block.write_output_samples(&mut writer).unwrap();
    //     if block.len() < block_size*channels as usize { break }
    // }
}
