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
        .filter(|fa| fa.frequency >= 20.0 && fa.frequency <= 20000.0)
        .collect();

    Model { spectrum: fa_data }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn frequency_to_color(value: f32) -> Rgb {
    let normalized = value.clamp(0.0, 1.0);

    Rgb::new(normalized, 0.0, 1.0 - normalized)
}

fn calculate_x_position(frequency: u32, window_width: f32) -> f32 {
    let min_freq = 20;
    let max_freq = 20000;

    let normalized_freq =
        (frequency as f32 - min_freq as f32) / (max_freq as f32 - min_freq as f32);

    normalized_freq * window_width - (window_width / 2.0)
}

fn calculate_y_position(fr_val: f32, window_height: f32) -> f32 {
    let normalized_val = fr_val.clamp(0.0, 1.0);
    normalized_val * window_height - (window_height / 2.0)
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let window_rect = app.window_rect();

    for fa in &model.spectrum {
        println!(
            "Drawing at frequency: {}, value: {}",
            fa.frequency, fa.amplitude
        );

        let color = frequency_to_color(fa.amplitude);

        let x_position = calculate_x_position(fa.frequency as u32, window_rect.w());
        let y_position = calculate_y_position(fa.amplitude, window_rect.h());

        draw.ellipse()
            .color(color)
            .x_y(x_position, y_position)
            .finish();
    }

    draw.to_frame(app, &frame).unwrap();
}
