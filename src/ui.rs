use std::sync::{Arc, Mutex};

use eframe::egui;
use eframe::egui::text_edit::TextEdit;
use eframe::egui::widgets::{Button, ProgressBar};
use eframe::egui::ColorImage;
use eframe::egui::Visuals;
use eframe::egui::{Color32, RichText};

use decoder;
use errors::DecoderError;

#[derive(PartialEq)]
enum DecoderRunState {
    RUNNING,
    CANCELED,
    DONE,
}

struct DecoderJobState {
    update_steps: u32,
    progress: f32,
    texture: Option<egui::TextureHandle>,
    run_state: DecoderRunState,
    error: Option<DecoderError>,
}

impl DecoderJobState {
    fn is_running(&self) -> bool {
        self.run_state == DecoderRunState::RUNNING
    }
}

impl Default for DecoderJobState {
    fn default() -> Self {
        Self {
            update_steps: 10,
            progress: 0.0,
            texture: None,
            run_state: DecoderRunState::DONE,
            error: None,
        }
    }
}

pub struct DecoderApp {
    input_path: String,
    output_path: String,
    decoding_state: Arc<Mutex<DecoderJobState>>,
}

impl DecoderApp {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        Self {
            input_path: input_path.to_owned(),
            output_path: output_path.to_owned(),
            decoding_state: Arc::new(Mutex::new(DecoderJobState::default())),
        }
    }
}

impl eframe::App for DecoderApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            input_path,
            output_path,
            decoding_state,
        } = self;

        {
            let mut state = decoding_state.lock().unwrap();

            if !ctx.input().raw.dropped_files.is_empty() && !state.is_running() {
                if let Some(path) = ctx.input().raw.dropped_files[0].clone().path {
                    *input_path = path.display().to_string();
                }
            }

            ctx.set_visuals(Visuals::dark());
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("APT-Decoder");

                egui::Grid::new("form_grid").num_columns(3).show(ui, |ui| {
                    ui.label("Input Wav File:");
                    ui.add_sized(
                        [300.0, 20.0],
                        TextEdit::singleline(input_path).interactive(!state.is_running()),
                    );

                    if ui
                        .add_enabled(!state.is_running(), Button::new("Open"))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            *input_path = path.display().to_string();
                        }
                    };
                    ui.end_row();

                    ui.label("Output PNG File:");
                    ui.add_sized(
                        [300.0, 20.0],
                        TextEdit::singleline(output_path).interactive(!state.is_running()),
                    );
                    if ui
                        .add_enabled(!state.is_running(), Button::new("Save"))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            *output_path = path.display().to_string();
                        }
                    };
                    ui.end_row();
                });

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!state.is_running(), Button::new("Decode"))
                        .clicked()
                    {
                        let ctx = ctx.clone();
                        let decoding_state = decoding_state.clone();
                        let input_path = input_path.clone();
                        let output_path = output_path.clone();

                        state.error = None;
                        state.run_state = DecoderRunState::RUNNING;
                        state.texture = None;

                        std::thread::spawn(move || {
                            let decoder_res =
                                decoder::decode(&input_path, &output_path, |progress, image| {
                                    let mut state = decoding_state.lock().unwrap();

                                    state.progress = progress;

                                    let size = [image.width() as _, image.height() as _];
                                    let color_img = ColorImage::from_rgba_unmultiplied(
                                        size,
                                        image.as_flat_samples().as_slice(),
                                    );

                                    state.texture =
                                        Some(ctx.load_texture("decoded-image", color_img));

                                    ctx.request_repaint();

                                    return (state.is_running(), state.update_steps);
                                });

                            let mut state = decoding_state.lock().unwrap();
                            state.run_state = DecoderRunState::DONE;
                            state.error = match decoder_res {
                                Err(err) => Some(err),
                                _ => None,
                            };

                            ctx.request_repaint();
                        });
                    }
                    if ui
                        .add_enabled(
                            state.is_running(),
                            Button::new(RichText::new("Cancel").color(Color32::RED)),
                        )
                        .clicked()
                    {
                        state.run_state = DecoderRunState::CANCELED;
                    }
                });

                let progressbar = ProgressBar::new(state.progress).show_percentage();
                ui.add(progressbar);
                ui.end_row();

                if let Some(err) = &state.error {
                    ui.label(RichText::new(err.to_string()).color(Color32::RED));
                };

                ui.separator();

                let image_size = ui.available_size();
                state.update_steps = image_size[1] as u32;

                if let Some(texture) = &state.texture {
                    ui.image(texture, image_size);
                }
            });
        }
    }
}
