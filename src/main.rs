use eframe::egui;
use eframe::egui::TextBuffer;
use sailii::config::Config;
use sailii::decoders::CrackResult;
use sailii::perform_cracking;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([680.0, 560.0])
            .with_title("sailii — Automatic Decryption"),
        ..Default::default()
    };
    eframe::run_native(
        "sailii",
        options,
        Box::new(|_cc| Ok(Box::new(SailiiApp::new()))),
    )
}

struct HistoryEntry {
    input: String,
    decoder: String,
    key: Option<String>,
    result: String,
}

enum WorkerMessage {
    Done(CrackResult, f64),
}

struct SailiiApp {
    input: String,
    result_output: String,
    result_decoder: String,
    result_key: String,
    result_label: String,
    result_visible: bool,
    history: Vec<HistoryEntry>,
    history_open: bool,
    is_running: bool,
    status_message: String,
    rx: Receiver<WorkerMessage>,
    worker_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl SailiiApp {
    fn new() -> Self {
        let (_, rx) = mpsc::channel();
        Self {
            input: String::new(),
            result_output: String::new(),
            result_decoder: String::new(),
            result_key: String::new(),
            result_label: String::new(),
            result_visible: false,
            history: Vec::new(),
            history_open: false,
            is_running: false,
            status_message: "Ready".to_string(),
            rx,
            worker_handle: Arc::new(Mutex::new(None)),
        }
    }

    fn start_decode(&mut self) {
        let input = self.input.trim().to_string();
        if input.is_empty() {
            self.status_message = "Please enter some text to decode".to_string();
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.rx = rx;
        self.is_running = true;
        self.result_visible = false;
        self.status_message = "Decoding...".to_string();

        let input_clone = input.clone();
        let config = Config {
            timeout_secs: 10,
            verbose: false,
            max_depth: 20,
            ..Default::default()
        };
        let start = std::time::Instant::now();

        let handle = thread::spawn(move || {
            let result = perform_cracking(&input_clone, config);
            let elapsed = start.elapsed().as_secs_f64();
            let msg = match result {
                Some(r) if r.success => WorkerMessage::Done(r, elapsed),
                _ => {
                    let mut r = CrackResult::new("None", "No decoder succeeded", "");
                    r.encrypted_text = input_clone;
                    WorkerMessage::Done(r, elapsed)
                }
            };
            let _ = tx.send(msg);
        });

        *self.worker_handle.lock().unwrap() = Some(handle);
    }
}

impl eframe::App for SailiiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMessage::Done(result, elapsed) => {
                    self.is_running = false;
                    if result.success {
                        let plaintext = result
                            .unencrypted_text
                            .as_ref()
                            .and_then(|v| v.first())
                            .cloned()
                            .unwrap_or_default();
                        self.result_output = plaintext.clone();
                        self.result_decoder = result.decoder.clone();
                        self.result_key = result.key.clone().unwrap_or_default();
                        self.result_label =
                            format!("Decoded via {} in {:.2}s", result.decoder, elapsed);
                        self.result_visible = true;
                        self.status_message = "Done".to_string();
                        self.history.push(HistoryEntry {
                            input: result.encrypted_text.clone(),
                            decoder: result.decoder.clone(),
                            key: result.key.clone(),
                            result: plaintext,
                        });
                    } else {
                        self.result_label = "Could not decode".to_string();
                        self.result_visible = true;
                        self.result_output.clear();
                        self.result_decoder.clear();
                        self.result_key.clear();
                        self.status_message = "Failed — no decoder matched".to_string();
                    }
                }
            }
        }

        if self.is_running {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("header")
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(24, 26, 32),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.heading(
                        egui::RichText::new("sailii")
                            .color(egui::Color32::from_rgb(88, 166, 255)),
                    );
                    ui.label(
                        egui::RichText::new("Automatic Decryption")
                            .color(egui::Color32::GRAY)
                            .size(14.0),
                    );
                });
                ui.add_space(10.0);
            });

        egui::TopBottomPanel::bottom("status")
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(30, 32, 38),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    if self.is_running {
                        ui.label(
                            egui::RichText::new("Decoding...")
                                .color(egui::Color32::YELLOW),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(&self.status_message)
                                .color(egui::Color32::GRAY),
                        );
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(18, 20, 24),
                ..Default::default()
            })
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(12.0);

                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(30, 32, 38))
                        .rounding(egui::Rounding::same(6.0))
                        .show(ui, |ui| {
                            ui.add_sized(
                                egui::vec2(ui.available_width(), 120.0),
                                egui::TextEdit::multiline(&mut self.input)
                                    .hint_text("Paste encoded text here...")
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        let decode_btn = egui::Button::new(egui::RichText::new("  Decode  ").size(14.0))
                            .fill(egui::Color32::from_rgb(88, 166, 255))
                            .rounding(egui::Rounding::same(4.0))
                            .min_size(egui::vec2(90.0, 32.0));

                        if ui.add_enabled(!self.is_running, decode_btn).clicked() {
                            self.start_decode();
                        }

                        if ui.button("Clear").clicked() {
                            self.input.clear();
                            self.result_visible = false;
                            self.result_output.clear();
                            self.result_decoder.clear();
                            self.status_message = "Cleared".to_string();
                        }

                        if ui.button("Paste").clicked() {
                            let text = ui.output_mut(|o| o.copied_text.take());
                            if !text.is_empty() {
                                self.input = text;
                                self.status_message = "Pasted".to_string();
                            }
                        }

                        let can_copy = self.result_visible && !self.result_output.is_empty();
                        if ui
                            .add_enabled(can_copy, egui::Button::new("Copy Result"))
                            .clicked()
                        {
                            ui.output_mut(|o| o.copied_text = self.result_output.clone());
                            self.status_message = "Copied to clipboard".to_string();
                        }
                    });

                    ui.add_space(12.0);

                    if self.result_visible {
                        let has_output = !self.result_output.is_empty();
                        let result_color = if has_output {
                            egui::Color32::from_rgb(24, 34, 26)
                        } else {
                            egui::Color32::from_rgb(40, 28, 28)
                        };
                        let border_color = if has_output {
                            egui::Color32::from_rgb(50, 130, 70)
                        } else {
                            egui::Color32::from_rgb(150, 60, 60)
                        };

                        egui::Frame::group(ui.style())
                            .fill(result_color)
                            .rounding(egui::Rounding::same(6.0))
                            .stroke(egui::Stroke::new(1.0, border_color))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&self.result_label)
                                        .color(if has_output {
                                            egui::Color32::from_rgb(160, 210, 170)
                                        } else {
                                            egui::Color32::from_rgb(220, 140, 140)
                                        })
                                        .size(12.0),
                                );
                                if has_output {
                                    ui.add_space(4.0);
                                    if !self.result_decoder.is_empty() {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "Decoder: {}",
                                                self.result_decoder
                                            ))
                                            .color(egui::Color32::LIGHT_GRAY)
                                            .size(12.0),
                                        );
                                    }
                                    if !self.result_key.is_empty() {
                                        ui.label(
                                            egui::RichText::new(format!("Key: {}", self.result_key))
                                                .color(egui::Color32::LIGHT_GRAY)
                                                .size(12.0),
                                        );
                                    }
                                    ui.add_space(6.0);
                                    egui::Frame::group(ui.style())
                                        .fill(egui::Color32::from_rgb(22, 24, 28))
                                        .rounding(egui::Rounding::same(4.0))
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                egui::vec2(ui.available_width(), 60.0),
                                                egui::TextEdit::multiline(&mut self.result_output)
                                                    .font(egui::TextStyle::Monospace)
                                                    .desired_width(f32::INFINITY)
                                                    .interactive(false),
                                            );
                                        });
                                }
                            });
                    }

                    ui.add_space(12.0);

                    if !self.history.is_empty() {
                        let open = &mut self.history_open;
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ctx,
                            egui::Id::new("history"),
                            *open,
                        )
                        .show_header(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("History  ({})", self.history.len()))
                                    .color(egui::Color32::LIGHT_GRAY)
                                    .size(13.0),
                            );
                        })
                        .body(|ui| {
                            egui::Frame::group(ui.style())
                                .fill(egui::Color32::from_rgb(26, 28, 34))
                                .rounding(egui::Rounding::same(4.0))
                                .show(ui, |ui| {
                                    let mut to_remove = None;
                                    for (i, entry) in self.history.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            let short_input = if entry.input.len() > 36 {
                                                format!("{}...", &entry.input[..33])
                                            } else {
                                                entry.input.clone()
                                            };
                                            let key_part = entry
                                                .key
                                                .as_ref()
                                                .map(|k| format!(", key={}", k))
                                                .unwrap_or_default();
                                            let short_result = if entry.result.len() > 28 {
                                                format!("{}...", &entry.result[..25])
                                            } else {
                                                entry.result.clone()
                                            };
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{}  →  {}  ({}{})",
                                                    short_input,
                                                    short_result,
                                                    entry.decoder,
                                                    key_part
                                                ))
                                                .color(egui::Color32::GRAY)
                                                .size(11.0)
                                                .monospace(),
                                            );
                                            if ui.small_button("x").clicked() {
                                                to_remove = Some(i);
                                            }
                                        });
                                    }
                                    if let Some(idx) = to_remove {
                                        self.history.remove(idx);
                                    }
                                });
                        });
                        // Save collapse state
                        *open = egui::collapsing_header::CollapsingState::load(
                            ctx,
                            egui::Id::new("history"),
                        )
                        .map(|s| s.is_open())
                        .unwrap_or(false);
                    }

                    ui.add_space(20.0);
                });
            });
    }
}
