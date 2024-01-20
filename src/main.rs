use hound;
use nannou::prelude::*;
use rustfft::{num_complex::Complex, FftPlanner};
use std::path::Path;

struct FrequencyAmplitude {
    frequency: f32,
    amplitude: f32,
}

struct Model {
    spectrum: Vec<FrequencyAmplitude>,
}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    let path = Path::new("src/audio/SampleAudio.wav");
    let mut reader = hound::WavReader::open(path).expect("Failed to open file");
    let spec = reader.spec();
    let min_freq = 20.0;
    let max_freq = 20000.0;

    let samples: Vec<i32> = reader.samples().map(|s| s.unwrap()).collect();

    let mut buffer: Vec<Complex<f32>> = samples
        .into_iter()
        .map(|x| Complex {
            re: x as f32,
            im: 0.0,
        })
        .collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(buffer.len());

    fft.process(&mut buffer);

    let fa_data: Vec<FrequencyAmplitude> = buffer
        .iter()
        .enumerate()
        .map(|(i, &complex_val)| {
            let amplitude = (complex_val.re.powi(2) + complex_val.im.powi(2)).sqrt();
            let frequency = i as f32 * spec.sample_rate as f32 / buffer.len() as f32;

            FrequencyAmplitude {
                frequency,
                amplitude,
            }
        })
        .filter(|fa| fa.frequency >= min_freq && fa.frequency <= max_freq)
        .collect();

    let compression_factor = 0.075;

    let compressed_fa_data: Vec<FrequencyAmplitude> = fa_data
        .iter()
        .map(|fa| FrequencyAmplitude {
            frequency: fa.frequency,
            amplitude: fa.amplitude.powf(compression_factor),
        })
        .collect();

    let max_compressed_amplitude = compressed_fa_data
        .iter()
        .map(|fa| fa.amplitude)
        .fold(0.0f32, f32::max);

    let normalized_fa_data: Vec<FrequencyAmplitude> = compressed_fa_data
        .into_iter()
        .map(|fa| {
            let normalized_frequency = (fa.frequency - min_freq) / (max_freq - min_freq);
            let normalized_amplitude = fa.amplitude / max_compressed_amplitude;

            FrequencyAmplitude {
                frequency: normalized_frequency,
                amplitude: normalized_amplitude,
            }
        })
        .collect();

    Model {
        spectrum: normalized_fa_data,
    }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn value_to_color(value: f32) -> Rgb {
    Rgb::new(value, 0.0, 1.0 - value)
}

fn position_by_value(value: f32, window_dimension: f32) -> f32 {
    value * window_dimension - (window_dimension / 2.0)
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let window_rect = app.window_rect();

    for fa in &model.spectrum {
        println!(
            "Drawing at frequency: {}, value: {}",
            fa.frequency, fa.amplitude
        );

        let color = value_to_color(fa.frequency);
        let x = position_by_value(fa.frequency, window_rect.w());
        let y = position_by_value(fa.amplitude, window_rect.h());

        draw.ellipse().color(color).x_y(x, y).finish();
    }

    draw.to_frame(app, &frame).unwrap();
}
