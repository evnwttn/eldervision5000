use hound;
use nannou::prelude::*;
use rustfft::{ num_complex::Complex, FftPlanner };
use std::path::Path;

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

struct Model {
    spectrum: FrequencyGroups,
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
    let compression_factor = 0.075;

    let samples: Vec<Complex<f32>> = reader
        .samples::<i32>()
        .map(|s| Complex::new(s.unwrap() as f32, 0.0))
        .collect();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(samples.len());
    let mut buffer = samples;
    fft.process(&mut buffer);
    let buffer_len = buffer.len();

    let compressed_fa: Vec<FrequencyAmplitudePair> = buffer
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i % 10000 == 0)
        .map(|(i, complex_val)| {
            let amplitude = complex_val.norm().powf(compression_factor); // Apply compression here
            let frequency = ((i as f32) * spec.sample_rate as f32) / buffer_len as f32;
            FrequencyAmplitudePair { frequency, amplitude }
        })
        .collect();

    let max_compressed_amplitude = compressed_fa.iter().map(|fa| fa.amplitude).fold(0.0, f32::max);

    let normalized_fa_data: Vec<FrequencyAmplitudePair> = compressed_fa
        .into_iter()
        .map(|fa| {
            let normalized_amplitude = fa.amplitude / max_compressed_amplitude;
            let frequency = (fa.frequency - min_freq) / (max_freq - min_freq);
            FrequencyAmplitudePair { frequency, amplitude: normalized_amplitude }
        })
        .filter(|fa| fa.frequency >= 0.0 && fa.frequency <= 1.0)
        .collect();

    let (mut low, mut mid, mut high) = (Vec::new(), Vec::new(), Vec::new());
    normalized_fa_data.into_iter().for_each(|fa| match fa.frequency {
        f if f <= 0.33 => low.push(fa),
        f if f <= 0.66 => mid.push(fa),
        _ => high.push(fa),
    });

    Model {
        spectrum: FrequencyGroups { low, mid, high },
    }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn value_to_blue(value: f32) -> Rgb {
    let blue = value.clamp(0.0, 1.0);
    Rgb::new(0.0, 0.0, blue)
}

fn value_to_orange(value: f32) -> Rgb {
    let orange_scale = value.clamp(0.0, 1.0);
    Rgb::new(1.0, 0.5 * orange_scale, 0.0)
}

fn value_to_yellow(value: f32) -> Rgb {
    let yellow_scale = value.clamp(0.0, 1.0);
    Rgb::new(1.0, 1.0, 0.0 * yellow_scale)
}

fn position_by_sine(value: f32, window_dimension: f32) -> f32 {
    (value * PI).sin() * window_dimension / 2.0
}

fn position_by_cosine(value: f32, window_dimension: f32) -> f32 {
    (value * PI).cos() * window_dimension / 2.0
}

fn position_by_tan(value: f32, window_dimension: f32) -> f32 {
    (value * PI).tan() * window_dimension / 2.0
}

fn position_by_value(value: f32, window_dimension: f32) -> f32 {
    value * window_dimension - window_dimension / 2.0
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let window_rect = app.window_rect();

    fn draw_frequency_amplitude_pair(
        draw: &Draw,
        fa: &FrequencyAmplitudePair,
        window_rect: Rect,
        color: Rgb,
        group: FrequencyGroup,
    ) {
        println!("Drawing at frequency: {}, amplitude: {}", fa.frequency, fa.amplitude);
        let x = match group {
            FrequencyGroup::Low => position_by_sine(fa.frequency, window_rect.w()),
            FrequencyGroup::Mid => position_by_cosine(fa.frequency, window_rect.w()),
            FrequencyGroup::High => position_by_tan(fa.frequency, window_rect.w()),
        };
        let y = position_by_value(fa.amplitude, window_rect.h());
        draw.ellipse().color(color).x_y(x, y).finish();
    }

    for fa in &model.spectrum.low {
        let color = value_to_blue(fa.frequency);
        draw_frequency_amplitude_pair(&draw, fa, window_rect, color, FrequencyGroup::Low);
    }

    for fa in &model.spectrum.mid {
        let color = value_to_orange(fa.frequency);
        draw_frequency_amplitude_pair(&draw, fa, window_rect, color, FrequencyGroup::Mid);
    }

    for fa in &model.spectrum.high {
        let color = value_to_yellow(fa.frequency);
        draw_frequency_amplitude_pair(&draw, fa, window_rect, color, FrequencyGroup::High);
    }

    draw.background().color(PLUM);

    draw.to_frame(app, &frame).unwrap();
}
