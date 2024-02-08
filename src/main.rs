use hound;
use nannou::prelude::*;
use rustfft::{ num_complex::Complex, FftPlanner };
use std::path::Path;

// Nannou uses a system roughly based on Model-View-Controller (MVC) pattern
// The program is split into a Model describing the internal state
// A View describing how to present the model and
// A Controller describing how to update the model on certain events

// Struct = user defined data type

struct FrequencyAmplitudePair {
    frequency: f32,
    amplitude: f32,
}

struct FrequencyGroups {
    low: Vec<FrequencyAmplitudePair>,
    mid: Vec<FrequencyAmplitudePair>,
    high: Vec<FrequencyAmplitudePair>,
}

enum FrequencyGroup {
    Low,
    Mid,
    High,
}

// Where we define app state
struct Model {
    spectrum: FrequencyGroups,
}

// Where a Rust program begins and ends
fn main() {
    nannou
        ::app(model) // Start building app & identify state
        .event(event) // Specify how app handles events
        .simple_window(view) // Request a window to which we will "draw" using view
        .run(); // Run the application
}

fn model(_app: &App) -> Model {
    // Reading .WAV audio file using library hound
    let path = Path::new("src/audio/SampleAudio.wav");
    let mut reader = hound::WavReader::open(path).expect("Failed to open file");

    // Retrieving file specs, in particular we are looking for Sample Rate for a frequency calculation
    // ie. 44.1kHz (44100 samples per second)
    let spec = reader.spec();

    // Identifying frequency range (hz)
    let min_freq = 60.0;
    let max_freq = 10000.0;

    // Starting the process of converting the audio samples read from the file
    // into a format (vector of complex numbers) necessary for performing a
    // Fast Fourier Transform (FFT), an algorithm used here to transform time based
    // audio signal data into a representation of frequency and amplitude

    let samples: Vec<Complex<f32>> = reader
        .samples::<i32>()
        .map(|s| Complex::new(s.unwrap() as f32, 0.0))
        .collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());
    let mut buffer = samples;
    fft.process(&mut buffer);
    let buffer_len = buffer.len();

    let fa_data: Vec<FrequencyAmplitudePair> = buffer
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i % 100 == 0)
        .map(|(i, complex_val)| {
            let amplitude = complex_val.norm();

            // Frequency calculation
            let frequency = ((i as f32) * (spec.sample_rate as f32)) / (buffer_len as f32);
            FrequencyAmplitudePair { frequency, amplitude }
        })
        .filter(|pair| pair.frequency >= min_freq && pair.frequency <= max_freq)
        .collect();

    // Normalize both frequency and amplitude values between 0.0 - 1.0

    let max_amplitude = fa_data
        .iter()
        .map(|pair| pair.amplitude)
        .fold(0.0f32, f32::max);

    let normalized_fa_data: Vec<FrequencyAmplitudePair> = fa_data
        .into_iter()
        .map(|pair| {
            let normalized_frequency = (pair.frequency - min_freq) / (max_freq - min_freq);

            // Normalize Amplitude and shift up by 1 decimal points, cap at 1.0
            let scaled_amplitude = (pair.amplitude / max_amplitude) * 10.0;
            let capped_amplitude = if scaled_amplitude > 1.0 { 1.0 } else { scaled_amplitude };

            FrequencyAmplitudePair {
                frequency: normalized_frequency,
                amplitude: capped_amplitude,
            }
        })
        .collect();

    // Splitting the data into three groups by frequency: low end, mid & high range
    // So I can represent each frequency band differently

    let (mut low, mut mid, mut high) = (Vec::new(), Vec::new(), Vec::new());
    normalized_fa_data.into_iter().for_each(|fa| {
        match fa.frequency {
            f if f <= 0.014 => low.push(fa), // Normalized 60 - 200 Hz range
            f if f <= 0.195 => mid.push(fa), // Normalized 200 - 2000 Hz range
            _ => high.push(fa), // Normalized 2000 - 10000 Hz range
        }
    });

    Model {
        spectrum: FrequencyGroups { low, mid, high },
    }
}

// This is where you'd include any events (controller layer)
// Such as window events (window movement, clicks, key presses, hovering, etc.)
// Device events, etc.
fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn amplitude_to_color(amplitude: f32, group: FrequencyGroup) -> Rgb {
    match group {
        FrequencyGroup::Low => { Rgb::new(0.0, 0.0, amplitude) }
        FrequencyGroup::Mid => { Rgb::new(1.0, 0.5 * amplitude, 0.0) }
        FrequencyGroup::High => { Rgb::new(1.0, 1.0, 0.0 * amplitude) }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let boundary = app.window_rect();

    for fa in &model.spectrum.low {
        let color = amplitude_to_color(fa.amplitude, FrequencyGroup::Low);
        let size = map_range(fa.amplitude, 0.0, 1.0, 10.0, 100.0);

        let x_value = fa.frequency * app.time;
        let y_value = (app.time / 2.0) * fa.frequency;

        let x = map_range(x_value, 0.0, 0.195, boundary.left(), boundary.right());
        let y = map_range(y_value, 0.0, 0.195, boundary.bottom(), boundary.top());

        draw.ellipse().color(color).x_y(x, y).w_h(size, size);
    }

    for fa in &model.spectrum.mid {
        let color = amplitude_to_color(fa.amplitude, FrequencyGroup::Mid);
        let size = map_range(fa.amplitude, 0.0, 1.0, 10.0, 100.0);

        let phase_shift = 2.0 * PI * fa.frequency;
        let x_value = (fa.frequency * app.time + phase_shift).sin();
        let y_value = ((app.time / 2.0) * fa.frequency + phase_shift).sin();

        let x = map_range(x_value, 0.0, 1.0, boundary.left(), boundary.right());
        let y = map_range(y_value, 0.0, 1.0, boundary.bottom(), boundary.top());

        draw.ellipse().color(color).x_y(x, y).w_h(size, size);
    }

    for fa in &model.spectrum.high {
        let color = amplitude_to_color(fa.amplitude, FrequencyGroup::High);
        let size = map_range(fa.amplitude, 0.0, 1.0, 10.0, 100.0);

        let x_value = (fa.frequency * app.time).sin();
        let y_value = (fa.frequency * (app.time / 2.0)).sin();

        let x = map_range(x_value, 0.0, 1.0, boundary.left(), boundary.right());
        let y = map_range(y_value, 0.0, 1.0, boundary.bottom(), boundary.top());

        draw.ellipse().color(color).x_y(x, y).w_h(size, size);
    }

    draw.background().color(BLACK);
    draw.to_frame(app, &frame).unwrap();
}
