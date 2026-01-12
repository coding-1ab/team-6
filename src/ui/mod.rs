use eframe::{App, Error, Frame, NativeOptions};
use egui::{Context, ViewportBuilder};
use rfd::FileDialog;

pub fn open_app() -> eframe::Result<(), Error> {
    let viewport = ViewportBuilder::default()
        .with_inner_size([1280.0, 720.0]);
    let options = NativeOptions {
        viewport,
        centered: true,
        persist_window: true,
        ..NativeOptions::default()
    };
    eframe::run_native("name", options, Box::new(|cc| {
        Ok(Box::new(Player {}))
    }))
}

struct Player {
}

impl App for Player {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("file", |ui| {
                    if ui.button("open").clicked() {
                        if let Some(path) = FileDialog::default().pick_file() {
                            let filename = path.file_name().unwrap().to_str().unwrap();
                            println!("{}", filename);
                        }
                    }

                    let sub2 = ui.button("sub 2");
                })
            })
        });
    }
}
