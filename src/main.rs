use std::{fs::File, io::Write};

mod ring_buffer;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

   let mut ring_buffer1 = ring_buffer::RingBuffer::<i16>::new(16);
   ring_buffer1.push(1);
   ring_buffer1.push(2);
   ring_buffer1.push(3);
   ring_buffer1.push(1);
   ring_buffer1.push(1);
   ring_buffer1.push(15);
   ring_buffer1.push(13);
   ring_buffer1.push(1);
   ring_buffer1.push(-21);
   ring_buffer1.push(13);
   ring_buffer1.push(-1);
   ring_buffer1.push(13);
   ring_buffer1.push(4);
   ring_buffer1.push(1);
   ring_buffer1.push(2);
   ring_buffer1.push(3);
   ring_buffer1.push(5); // Will not happen
   println!("{}", ring_buffer1.len());
   println!("{}", ring_buffer1.pop());
   println!("{}", ring_buffer1.pop());
   println!("{}", ring_buffer1.pop());
   println!("{}", ring_buffer1.pop());
   println!("{:?}", ring_buffer1);
   ring_buffer1.push(47);
   ring_buffer1.push(48);
   println!("{}", ring_buffer1.pop());
   println!("{:?}", ring_buffer1);



    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output text filename>", args[0]);
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels;

    // Read audio data and write it to the output text file (one column per channel)
    let mut out = File::create(&args[2]).expect("Unable to create file");
    for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        write!(out, "{}{}", sample, if i % channels as usize == (channels - 1).into() { "\n" } else { " " }).unwrap();
    }
    // println!("{}", char::default());
}
