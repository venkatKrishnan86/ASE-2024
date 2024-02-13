use std::{fs::File, io::Write};

mod comb_filter;

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
    let channels: u16 = spec.channels;
    // const channels: u16 = 2;

    // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
    //       Use the following block size:
    const block_size: usize = 1024;

    // Read audio data and write it to the output text file (one column per channel)
    let mut out: File = File::create(&args[2]).expect("Unable to create file");
    // for (i, sample) in reader.samples::<i16>().enumerate() {
    //     let sample = sample.unwrap() as f32 / (1 << 15) as f32;
    //     write!(out, "{}{}", sample, if i % channels as usize == (channels - 1).into() { "\n" } else { " " }).unwrap();
    // }
    let mut comb_filter_1 = comb_filter::CombFilter::new(comb_filter::FilterType::FIR, 0.05, 44100.0, channels as usize);
    let mut channel_output_values: Vec<String> = Vec::new();
    for _ in 0..channels { 
        channel_output_values.push(String::from(""));
    }

    while let Ok(block) = reader.samples::<i16>().take(block_size*channels as usize).collect::<Result<Vec<_>, _>>() {
        
        let mut real_input_block: Vec<[f32; block_size]> = vec![[0.0; block_size]; channels as usize];
        let mut input_block_immut: Vec<&[f32]> = Vec::new();

        let mut real_output_block: Vec<[f32; block_size]> = vec![[0.0; block_size]; channels as usize];
        let mut output_block_mut: Vec<&mut [f32]> = Vec::new();

        for (i, &sample) in block.iter().enumerate() {
            let sample = sample as f32 / (1 << 15) as f32;
            let channel = i % channels as usize;
            let index = i / channels as usize;
            real_input_block[channel][index] = sample;
        }

        for (input_blocks, output_blocks) in real_input_block.iter().zip(real_output_block.iter_mut()) {
            input_block_immut.push(input_blocks);
            output_block_mut.push(output_blocks);
        }

        comb_filter_1.process(&input_block_immut, &mut output_block_mut);

        for (i, output_block) in real_output_block.into_iter().enumerate() {
            for value in output_block {
                channel_output_values[i].push_str(&format!("{}\n", value));
            }
        }

        if block.len() < block_size*channels as usize {
            break
        }
    }
    for channel_sample in channel_output_values {
        write!(out, "{}", channel_sample);
    }
    // write!(out, "{}{}", sample, if i % channels as usize == (channels - 1).into() { "\n" } else { " " }).unwrap();
}
