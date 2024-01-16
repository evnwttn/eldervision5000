use audrey::read::Reader;
use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::fs::File;
use std::path::Path;

fn main() {
    let path = Path::new("src/audio/Alien Space Bats.wav");
    let file = File::open(path).expect("Failed to open file");

    let mut reader = Reader::new(file).expect("Failed to create reader");
    let desc = reader.description();
    println!("Audio format: {:?}", desc.format());

    let samples: Vec<f32> = reader
        .samples::<f32>()
        .take(2048) // Taking 2048 samples; adjust as needed
        .map(|s| s.expect("Error reading sample"))
        .collect();

    let hann_window = hann_window(&samples);

    let spectrum_hann_window = samples_fft_to_spectrum(
        &hann_window,
        44100,
        FrequencyLimit::Range(20.0, 20000.0),
        Some(&divide_by_N_sqrt),
    )
    .unwrap();

    for (fr, fr_val) in spectrum_hann_window.data().iter() {
        println!("{}Hz => {}", fr, fr_val)
    }
}
