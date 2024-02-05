use std::sync::{Arc, Mutex};

use egui_plot::{Line, Plot};

use crate::{FFTPlot, TimeSeriesPlot};

pub struct PlotApp {
    pub timeseries: Arc<Mutex<TimeSeriesPlot>>,
    pub fft: Arc<Mutex<FFTPlot>>,
    y_max: f64,
    f_max: f64,
    history_s: f64,
}

impl PlotApp {
    /// Called once before the first frame.
    pub fn new() -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let f_max = 75.0;
        let history_s = 10.0;
        let fs = 1000.0;

        Self {
            timeseries: Arc::new(Mutex::new(TimeSeriesPlot::new(fs, history_s))),
            fft: Arc::new(Mutex::new(FFTPlot::new(f_max))),
            y_max: 1.0,
            f_max,
            history_s,
        }
    }
}

impl eframe::App for PlotApp {
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        let mut new_fmax = self.f_max;
        let mut new_hist = self.history_s;
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);

                ui.add(egui::Label::new("Ymax:"));
                ui.add(egui::widgets::DragValue::new(&mut self.y_max).clamp_range(0..=100));

                ui.add(egui::Label::new("Fmax:"));
                ui.add(egui::widgets::DragValue::new(&mut new_fmax).clamp_range(5..=1000));

                ui.add(egui::Label::new("Hist:"));
                ui.add(egui::widgets::DragValue::new(&mut new_hist).clamp_range(2..=600));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            let y_max = self.y_max;
            let time_plot = Plot::new("Time Plot").view_aspect(4.0);
            time_plot
                .include_y(y_max)
                .include_y(-y_max)
                .show(ui, |plot_ui| {
                    plot_ui.line(Line::new(self.timeseries.lock().unwrap().plot_values()));
                });

            ui.separator();

            let fft_plot = Plot::new("fft").view_aspect(4.0).include_x(self.f_max);
            fft_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(self.fft.lock().unwrap().plot_values()));
            });

            ctx.request_repaint();
        });

        if self.f_max != new_fmax {
            self.f_max = new_fmax;
            self.fft.lock().unwrap().set_f_max(new_fmax);
        }
        if self.history_s != new_hist {
            self.history_s = new_hist;
            self.timeseries.lock().unwrap().update_history_s(new_hist);
        }
    }
}
