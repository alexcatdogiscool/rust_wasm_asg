
use std::f32::consts::TAU;
use std::f32::consts::PI;

use hound;
use rustfft::{FftPlanner, num_complex::Complex};
use std::env;



const BAUD_RATE: usize = 10;
const SAMPLE_RATE: usize = 44100;
const SAMPLES_PER_BIT: usize = SAMPLE_RATE / BAUD_RATE;

fn string_to_bin(data: String) -> String {
    let binary = data.bytes().map(|b| format!("{:08b}", b)).collect::<Vec<String>>().join(" ");
    binary
}

fn encode_bit(freq: f32, duration: f32) -> Vec<f32> {
    let mut data: Vec<f32> = Vec::new();
    let num_samples = SAMPLES_PER_BIT;
    let period = SAMPLE_RATE as f32 / freq;

    for i in 0..num_samples {
        let amp = (TAU * freq * i as f32 / SAMPLE_RATE as f32).sin();
        data.push(amp);
    }
    data
}

fn encode(data: String) -> Vec<f32> {
    let mut audio: Vec<f32> = Vec::new();
    let bin = string_to_bin(data);
    let mut bit: Vec<f32> = Vec::new();

    // preamble
    bit = encode_bit(2000.0, 1.0 / BAUD_RATE as f32);
    audio.append(&mut bit);
    
    // data
    
    for c in bin.chars() {

        let mut bit = match c {
            '1' => encode_bit(1100.0, 1.0 / BAUD_RATE as f32),
            '0' => encode_bit(1000.0, 1.0 / BAUD_RATE as f32),
            _ => continue,
            
        };
        audio.append(&mut bit);
    }

    // post-amble(?)
    bit = encode_bit(2000.0, 1.0 / BAUD_RATE as f32);
    audio.append(&mut bit);

    audio
}

fn write_to_file(samples: Vec<f32>, path: &str) -> Result<(), hound::Error> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(path, spec).unwrap();

    for s in samples {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(())
}

fn dominant_freq(samples: &[f32]) -> [(f32, f32); 2] {
    let n = samples.len();
    let mut buffer: Vec<Complex<f32>> = samples.iter().map(|&x| Complex{ re: x, im: 0.0 }).collect();
    apply_hanning(&mut buffer);
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    fft.process(&mut buffer);

    let mut freqs: Vec<(f32, f32)> = buffer[..n/2].iter()
        .enumerate()
        .map(|(i, c)| (i as f32 * SAMPLE_RATE as f32 / n as f32, c.norm()))
        .collect();

    freqs.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());

    // return loadest 2 frequencies and their magnitudes
    return [freqs[0], freqs[1]];
    // this was a pain!!!
}

fn apply_hanning(samples: &mut [Complex<f32>]) {
    let n = samples.len();
    for i in 0..n {
        let w = 0.5 - 0.5 * (TAU * i as f32 / n as f32).cos();
        samples[i].re *= w;
    }
    // cool freq domain filter!!
}


fn decode(audio: Vec<f32>, is_aligned: bool) -> Vec<char> {
    let hop = SAMPLES_PER_BIT;
    let mut bits: Vec<char> = Vec::new();

    for s in (0..audio.len() - SAMPLES_PER_BIT).step_by(hop) {
        let freq_tup = dominant_freq(&audio[s..s+SAMPLES_PER_BIT]);
        let freq = freq_tup[0].0;
        //println!("{}", freq);

        let is_preamble = freq_tup[0].0 > 1800.0 && freq_tup[0].0 < 2200.0;

        if (is_preamble) {
            //found preamble!!! (or postamble)
            if (is_aligned) {
                // postamble
                println!("end of data");
                return bits;
            }
            println!("found preamble");
            let data_start = s + (hop*2);  // just start after this window
            if data_start < audio.len() {
                return decode(audio[data_start..].to_vec(), true);
            }
            continue;
        }

        let mut bit: Option<char> = None;
        if (freq > 800.0 && freq < 1050.0) {
            //println!("0");
            bit = Some('0');
        } else if (freq > 1050.0 && freq < 1300.0) {
            //println!("1");
            bit = Some('1');
        }

        if let Some(b) = bit {
            bits.push(b);
        }
        
        
        
    }

    bits
}

fn read_from_file(path: &str) -> Vec<f32> {
    let mut reader = hound::WavReader::open(path).unwrap();
    let spec = reader.spec();
    let channels = spec.channels as usize;

    let raw: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().filter_map(|s| s.ok()).collect()
        }
        hound::SampleFormat::Int => {
            let max_val = (1_i32 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max_val)
                .collect()
        }
    };

    // average every `channels` samples into one mono sample
    raw.chunks(channels)
        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
        .collect()
}

fn bin_to_string(bits: Vec<char>) -> String {
    bits.chunks(8).filter_map(|chunk| {
        let byte: String = chunk.iter().collect();
        u8::from_str_radix(&byte, 2).ok()
    })
    .map(|b| b as char)
    .collect()
}

fn main() {


    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Usage:\ncmd enc input file_name | to encode 'input' and save audio to 'file_name'");
        println!("or");
        println!("cmd dec file_name | to decode the data in 'file_name");
        return;
    }

    match args[1].as_str() {
        "enc" => {
            if args.len() < 4 {
                println!("Usage: cmd enc input file_name");
                return;
            }
            let input = args[2].clone();
            let path = args[3].clone();
            let audio = encode(input);
            match write_to_file(audio, &path) {
                Ok(_) => println!("writen to {}", path),
                Err(e) => println!("error writing file: {}", e),
            }
        }

        "dec" => {
            if args.len() < 3 {
                println!("Usage: cmd dec file_name");
                return;
            }
            let path = args[2].clone();
            let samples = read_from_file(&path);
            let decoded = decode(samples, false);
            let out = bin_to_string(decoded);
            println!("{}", out);

        }
        _ => { println!("operation needs to be [enc|dec]") }

    }



}
