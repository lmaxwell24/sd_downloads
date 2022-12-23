use std::{fs, path::PathBuf};

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct App {
    output_loc: PathBuf,
    transferred_files: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            output_loc: PathBuf::from("./"),
            transferred_files: Vec::new()
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                if ui.button("Change output path").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.output_loc = path;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag and drop folder/files to upload");

            if !ctx.input().raw.dropped_files.is_empty() {
                for file in ctx.input().raw.dropped_files.clone() {
                    let contents = if let Some(path) = &file.path {
                        let pathbufifiedpath = PathBuf::from(path);
                        if path.is_dir() {
                            compatible_files_in_path(pathbufifiedpath)
                        } else {
                            vec![pathbufifiedpath]
                        }
                    } else {
                        Vec::new()
                    };

                    for file in contents {
                        println!("{:?}", file);
                        let filename = file.file_name().unwrap();
                        if let Err(err) = fs::copy(&file, &self.output_loc.join(filename)) {
                            println!("{:?}", err);
                        }else {
                            let successful_name = filename.to_str().unwrap();
                            self.transferred_files.push(successful_name.to_string());
                        }
                    }
                }
            }
            for file in self.transferred_files.iter() {
                ui.label(format!("{}", file));
            }
        });
    }
}

fn compatible_files_in_path(p: PathBuf) -> Vec<PathBuf> {
    if p.is_dir() {
        let mut retvec: Vec<PathBuf> = Vec::new();
        for file in p.read_dir().expect("this should not error") {
            if let Ok(entry) = file {
                if entry.path().is_dir() {
                    let mut others = compatible_files_in_path(entry.path());
                    retvec.append(&mut others);
                }
                if compatible_extension(&entry.path()) {
                    retvec.push(entry.path());
                }
            }
        }
        retvec
    } else {
        if compatible_extension(&p) {
            vec![p]
        } else {
            Vec::new()
        }
    }
}

const COMPAT_EXTENSIONS: &[&str] = &["CR2", "JPG"];

fn compatible_extension(p: &PathBuf) -> bool {
    if let Some(ext) = p.extension() {
        return COMPAT_EXTENSIONS.contains(&(ext.to_ascii_uppercase().to_str().unwrap()));
    }
    false
}
