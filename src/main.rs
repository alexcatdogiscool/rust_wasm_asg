
use std::f32::consts::TAU;
use std::f32::consts::PI;

use hound;
use rustfft::{FftPlanner, num_complex::Complex};



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
        if c == '1' {
            bit = encode_bit(1100.0, 1.0 / BAUD_RATE as f32);
        }
        else if c == '0' {
            bit = encode_bit(1000.0, 1.0 / BAUD_RATE as f32);
        }
        else {
            //bit = encode_bit(500.0, 1.0 / BAUD_RATE as f32);
            //nothing
        }
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

        if (((freq_tup[0].0 > 1800.0 && freq_tup[0].0 < 2200.0) && (freq_tup[1].0 > 800.0 && freq_tup[1].0 < 1300.0)) || ((freq_tup[1].0 > 1800.0 &&freq_tup[1].0 < 2200.0) && (freq_tup[0].0 > 800.0 && freq_tup[0].0 < 1300.0))) {// evil if statement
            // if (loudest ton is 2khz and second loadest is 1khz) OR
            // (loadest tone in 1khz and second loadest in 2khz)
            
            
            //found preamble!!! (or postamble)
            if (is_aligned) {
                // postamble
                println!("end of data");
                return bits;
            }
            // we are in preamble
            // need to align with it
            // either out hop starts in the preamble and overflows to data, or
            // starts in noise and overflows into preamble.
            if (freq_tup[1].0 > 800.0 && freq_tup[1].0 < 1050.0) {
                // the second loadest tone is 1khz
                let ratio = freq_tup[0].1 / freq_tup[1].1;
                // we are "ratio" of the way into preamble
                let offset = SAMPLES_PER_BIT as f32 * ratio;
                let data_start = ((s+hop) - offset as usize);
                return decode(audio[data_start..].to_vec(), true);
                // call this func again but at the starting pos.
                // this preamble code wont be called till the postamble
            }
        }

        let mut bit: Option<char> = None;
        if (freq > 800.0 && freq < 1050.0) {
            bit = Some('0');
        } else if (freq > 1050.0 && freq < 1300.0) {
            bit = Some('1');
        }

        if let Some(b) = bit {
            bits.push(b);
        } else {
            println!("bit was None: {}, {}", freq, bits.len());
        }
        
        
        
    }

    bits
}


fn main() {
    

    let s = "hello".to_string();
    println!("{}", s);
    let b = string_to_bin(s.clone());
    println!("{}", b);

    let audio = encode(s);

    let decoded = decode(audio, false);
    let out: String = decoded.into_iter().collect();

    println!("{}", out);

}
