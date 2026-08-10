/* FastNote Rust/egui Edition */

use std::fs;
use std::path::PathBuf;

struct FastNoteApp {
    notes_dir: String,
    current_path: Option<PathBuf>,
    document_content: String,
    dirty: bool,
    show_preview: bool,
    message: Option<String>,
}

impl FastNoteApp {
    fn new() -> Self {
        Self {
            notes_dir: std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
            current_path: None,
            document_content: String::new(),
            dirty: false,
            show_preview: false,
            message: None,
        }
    }

    fn open_file(&mut self, path: &str) {
        match fs::read_to_string(path) {
            Ok(content) => {
                self.document_content = content;
                self.current_path = Some(path.into());
                self.dirty = false;
            }
            Err(e) => self.message = Some(format!("Cannot open: {}", e)),
        }
    }

    fn save_file(&mut self) {
        if let Some(ref path) = self.current_path {
            if let Err(e) = fs::write(path, &self.document_content) {
                self.message = Some(format!("Save failed: {}", e));
            } else {
                self.dirty = false;
                self.message = Some(format!("Saved: {}", path.display()));
            }
        } else {
            self.message = Some("No file open".into());
        }
    }

    fn export_html(&mut self, output: &str) {
        let html = render_markdown(&self.document_content);
        if let Err(e) = fs::write(output, &html) {
            self.message = Some(format!("Export failed: {}", e));
        } else {
            self.message = Some(format!("Exported: {}", output));
        }
    }
}

fn render_markdown(md: &str) -> String {
    let escaped = md.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let mut html = String::from("<!DOCTYPE html><html><body>\n");
    for line in escaped.lines() {
        if line.is_empty() {
            html.push_str("<br>\n");
        } else if let Some(s) = line.strip_prefix("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", s));
        } else if let Some(s) = line.strip_prefix("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", s));
        } else if line.starts_with("**") && line.ends_with("**") && line.len() > 4 {
            html.push_str(&format!("<strong>{}</strong>\n", &line[2..line.len() - 2]));
        } else if line.starts_with("- ") {
            html.push_str(&format!("<li>{}</li>\n", &line[2..]));
        } else {
            html.push_str(&format!("<p>{}</p>\n", line));
        }
    }
    html.push_str("</body></html>\n");
    html
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version") {
        println!("fastnote-rust-egui v1.0");
        return;
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FastNote",
        options,
        Box::new(|_cc| Ok(Box::new(FastNoteApp::new()))),
    );
}

impl eframe::App for FastNoteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle headless args
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--headless") {
            let mut i = 1;
            while i < args.len() {
                if args[i] == "--selftest" {
                    let html = render_markdown("# Hello\n**World**");
                    if html.contains("<h1>") && html.contains("<strong>") {
                        println!("selftest: pass");
                    } else {
                        println!("selftest: fail");
                    }
                    std::process::exit(0);
                }
                i += 1;
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.open_file(path.to_str().unwrap());
                        }
                    }
                    if ui.button("Save").clicked() {
                        self.save_file();
                    }
                    if ui.button("Export HTML").clicked() {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            self.export_html(path.to_str().unwrap());
                        }
                    }
                });
                ui.separator();
                ui.checkbox(&mut self.show_preview, "Preview");
                if self.dirty {
                    ui.label(" *");
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref msg) = self.message {
                ui.label(egui::RichText::new(msg).color(egui::Color32::YELLOW));
                self.message = None;
            }

            if self.show_preview {
                let html = render_markdown(&self.document_content);
                ui.add(egui::TextEdit::multiline(&mut String::from(&html)).desired_rows(20));
            } else {
                ui.add(
                    egui::TextEdit::multiline(&mut self.document_content)
                        .code_editor()
                        .desired_rows(20),
                );
                if ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
                    self.save_file();
                }
            }
        });

        ctx.request_repaint();
    }
}
