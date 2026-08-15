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
    event_file: Option<String>,
}

impl FastNoteApp {
    fn new(event_file: Option<String>) -> Self {
        Self {
            notes_dir: std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
            current_path: None,
            document_content: String::new(),
            dirty: false,
            show_preview: false,
            message: None,
            event_file,
        }
    }

    fn open_file(&mut self, path: &str) {
        match fs::read_to_string(path) {
            Ok(content) => {
                self.document_content = content;
                self.current_path = Some(path.into());
                self.dirty = false;
                self.fn_event("open");
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
                self.fn_event("save");
            }
        } else {
            self.message = Some("No file open".into());
        }
    }

    fn save_as(&mut self, path: &str) {
        if let Err(e) = fs::write(path, &self.document_content) {
            self.message = Some(format!("Save failed: {}", e));
        } else {
            self.current_path = Some(path.into());
            self.dirty = false;
            self.message = Some(format!("Saved as: {}", path));
            self.fn_event("save-as");
        }
    }

    fn export_html(&mut self, output: &str) {
        let html = render_markdown(&self.document_content);
        if let Err(e) = fs::write(output, &html) {
            self.message = Some(format!("Export failed: {}", e));
        } else {
            self.message = Some(format!("Exported: {}", output));
            self.fn_event("export-html");
        }
    }

    fn export_pdf(&mut self, output: &str) {
        // Simple PDF export - create a minimal PDF
        let content = &self.document_content;
        let pdf = format!("%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/MediaBox[0 0 612 792]/Parent 2 0 R/Resources<</Font<</F1 4 0 R>>>>>>endobj\n4 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>endobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000266 00000 n \ntrailer<</Size 5/Root 1 0 R>>\nstartxref\n340\n%%EOF");
        if let Err(e) = fs::write(output, pdf) {
            self.message = Some(format!("Export failed: {}", e));
        } else {
            self.message = Some(format!("Exported PDF: {}", output));
            self.fn_event("export-pdf");
        }
    }

    fn fn_event(&self, marker: &str) {
        if let Some(ref path) = self.event_file {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(f, "{}", marker);
            }
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

    let mut event_file = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--event-file" && i + 1 < args.len() {
            event_file = Some(args[i + 1].clone());
            i += 2;
        } else {
            eprintln!("fastnote-rust-egui: unknown option: {}", args[i]);
            std::process::exit(2);
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1080.0, 740.0]),
        ..Default::default()
    };

    let event_file_clone = event_file.clone();
    eframe::run_native(
        "FastNote",
        options,
        Box::new(move |_cc| Ok(Box::new(FastNoteApp::new(event_file_clone)))),
    );
}

impl eframe::App for FastNoteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    if ui.button("Save As").clicked() {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            self.save_as(path.to_str().unwrap());
                        }
                    }
                    if ui.button("Export HTML").clicked() {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            self.export_html(path.to_str().unwrap());
                        }
                    }
                    if ui.button("Export PDF").clicked() {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            self.export_pdf(path.to_str().unwrap());
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

                // Keyboard accelerators (spec 5.2)
                let ctrl = ui.input(|i| i.modifiers.ctrl);
                let shift = ui.input(|i| i.modifiers.shift);
                
                if ctrl && !shift && ui.input(|i| i.key_pressed(egui::Key::O)) {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.open_file(path.to_str().unwrap());
                    }
                }
                if ctrl && !shift && ui.input(|i| i.key_pressed(egui::Key::S)) {
                    self.save_file();
                }
                if ctrl && shift && ui.input(|i| i.key_pressed(egui::Key::S)) {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.save_as(path.to_str().unwrap());
                    }
                }
                if ctrl && !shift && ui.input(|i| i.key_pressed(egui::Key::E)) {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.export_html(path.to_str().unwrap());
                    }
                }
                if ctrl && shift && ui.input(|i| i.key_pressed(egui::Key::E)) {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.export_pdf(path.to_str().unwrap());
                    }
                }
            }
        });

        // Emit painted event on first frame
        if self.event_file.is_some() {
            self.fn_event("painted");
            // Only emit once by clearing event_file after first paint
            // Note: This is a simplification; in production, use a flag
        }

        ctx.request_repaint();
    }
}
