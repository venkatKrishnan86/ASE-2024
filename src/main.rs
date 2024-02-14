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
    let mut reader = hound::WavReader::open(&args[1]).expect("Could not read file");
    let spec = reader.spec();
    let channels: usize = spec.channels as usize;

    if spec.bits_per_sample!=16 {
        eprintln!("Bit depth must be 16 bit! Bit depth of the current song: {}", spec.bits_per_sample);
        return
    }

    // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
    //       Use the following block size:
    let block_size: usize = 1024;

    comb_filter::process_and_write_audio(
        &mut reader, 
        block_size, 
        channels,
        &args[2], 
        spec, 
        comb_filter::FilterType::IIR, 
        0.8
    );
}