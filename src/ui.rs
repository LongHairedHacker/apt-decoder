use std::sync::{Arc, Mutex};
use std::time;

use eframe::egui::text_edit::TextEdit;
use eframe::egui::widgets::{Button, ProgressBar};
use eframe::egui::{Color32, RichText};
use eframe::{egui, epi};

use decoder;

#[derive(PartialEq)]
enum DecoderRunState {
    RUNNING,
    CANCELED,
    DONE,
}

struct DecoderJobState {
    progress: f32,
    image: Option<image::GrayImage>,
    run_state: DecoderRunState,
}

impl DecoderJobState {
    fn is_running(&self) -> bool {
        self.run_state == DecoderRunState::RUNNING
    }
}

impl Default for DecoderJobState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            image: None,
            run_state: DecoderRunState::DONE,
        }
    }
}

pub struct DecoderApp {
    input_path: String,
    output_path: String,
    decoding_state: Arc<Mutex<DecoderJobState>>,
    texture: Option<(egui::TextureId, egui::Vec2)>,
}

impl Default for DecoderApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            input_path: "input.wav".to_owned(),
            output_path: "output.png".to_owned(),
            decoding_state: Arc::new(Mutex::new(DecoderJobState::default())),
            texture: None,
        }
    }
}

impl epi::App for DecoderApp {
    fn name(&self) -> &str {
        "eframe template"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let Self {
            input_path,
            output_path,
            decoding_state,
            texture,
        } = self;

        {
            let mut state = decoding_state.lock().unwrap();

            if !ctx.input().raw.dropped_files.is_empty() && !state.is_running() {
                if let Some(path) = ctx.input().raw.dropped_files[0].clone().path {
                    *input_path = path.display().to_string();
                }
            }

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
                        let frame = frame.clone();
                        let decoding_state = decoding_state.clone();
                        let input_path = input_path.clone();
                        let output_path = output_path.clone();
                        state.run_state = DecoderRunState::RUNNING;
                        std::thread::spawn(move || {
                            decoder::decode(&input_path, &output_path, |progress, image| {
                                let mut state = decoding_state.lock().unwrap();

                                state.progress = progress;
                                state.image = Some(image);

                                frame.request_repaint();

                                return state.is_running();
                            })
                            .unwrap();

                            let mut state = decoding_state.lock().unwrap();
                            state.run_state = DecoderRunState::DONE;
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

                ui.separator();

                if let Some(image) = state.image.take() {
                    let image = image::DynamicImage::ImageLuma8(image);
                    let size = [image.width() as _, image.height() as _];
                    let pixels = image.into_rgba8();
                    let epi_img = epi::Image::from_rgba_unmultiplied(
                        size,
                        pixels.as_flat_samples().as_slice(),
                    );
                    let size = egui::Vec2::new(size[0] as f32, size[1] as f32);

                    if let Some((old_texture, _)) = texture {
                        frame.free_texture(*old_texture);
                    }

                    *texture = Some((frame.alloc_texture(epi_img), size));

                    state.image = None;
                }

                if let Some((texture, size)) = texture {
                    ui.image(*texture, *size);
                }
            });
        }
    }
}
