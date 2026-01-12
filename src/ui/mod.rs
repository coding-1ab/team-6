use eframe::{App, Error, Frame, NativeOptions};
use egui::{Context, FontData, FontFamily, ViewportBuilder, epaint::text::{FontInsert, FontPriority, InsertFontFamily}};
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
        let nanum_font = include_bytes!("../../NanumGothic.ttf");
        let font_data = FontData::from_static(nanum_font);
        let font = FontInsert {
            name: "Nanum Gothic".to_string(),
            data: font_data,
            families: vec![InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Highest,
            }],
        };
        cc.egui_ctx.add_font(font);
        Ok(Box::new(Player {
            open_file_name: String::new()
        }))
    }))
}

struct Player {
    open_file_name: String,
}

impl App for Player {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("file", |ui| {
                    if ui.button("open").clicked() {
                        if let Some(path) = FileDialog::default().pick_file() {
                            let file_name = path.file_name().unwrap();
                            self.open_file_name = String::from(file_name.to_str().unwrap());
                        }
                    }

                    let sub2 = ui.button("sub 2");
                })
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.open_file_name);
        });
    }
}
