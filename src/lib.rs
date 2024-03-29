#![warn(clippy::all, rust_2018_idioms)]

mod app;
use std::collections::VecDeque;

pub use app::PlotApp;

pub type Measurement = egui_plot::PlotPoint;

pub struct TimeSeriesPlot {
    pub values: VecDeque<Measurement>,
    pub max_points: usize,
    pub fs: f64,
}

impl TimeSeriesPlot {
    pub fn new(fs: f64, history_seconds: f64) -> Self {
        let max_points = (fs * history_seconds) as usize;
        Self {
            values: VecDeque::with_capacity(max_points),
            max_points,
            fs,
        }
    }

    /// Set the number of seconds of historical data that wil be shown.
    pub fn update_history_s(&mut self, num_seconds: f64) {
        self.max_points = (self.fs * num_seconds) as usize;
    }

    pub fn add(&mut self, t: f64, val: f64) {
        if let Some(last) = self.values.back() {
            if t < last.x {
                self.values.clear()
            }
        }

        let measurement = Measurement { x: t, y: val };

        self.values.push_back(measurement);

        let limit = self.values.back().unwrap().x - (self.max_points as f64);
        while let Some(front) = self.values.front() {
            if front.x >= limit {
                break;
            }
            self.values.pop_front();
        }
    }

    pub fn plot_values(&self) -> egui_plot::PlotPoints {
        egui_plot::PlotPoints::Owned(Vec::from_iter(self.values.iter().copied()))
    }
}

pub struct FFTPlot {
    pub values: Vec<Measurement>,
    pub max_freq: f64,
}

impl FFTPlot {
    pub fn new(max_freq: f64) -> Self {
        Self {
            values: Vec::new(),
            max_freq,
        }
    }

    pub fn set_f_max(&mut self, max_freq: f64) {
        self.max_freq = max_freq;
    }

    pub fn add(&mut self, values: Vec<f64>) {
        let n = values.len();
        // generate frequecny axis
        let freqs: Vec<f64> = (0..n)
            .map(|i| i as f64 * self.max_freq / n as f64)
            .collect();

        // Create measurement vector by zipping freqs and values
        self.values = freqs
            .into_iter()
            .zip(values.into_iter())
            .map(|(x, y)| Measurement { x, y })
            .collect();
    }

    pub fn plot_values(&self) -> egui_plot::PlotPoints {
        egui_plot::PlotPoints::Owned(Vec::from_iter(self.values.iter().copied()))
    }
}
