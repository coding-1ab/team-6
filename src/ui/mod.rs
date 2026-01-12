use eframe::{App, Error, Frame, NativeOptions};
use egui::{Context, FontData, FontFamily, ViewportBuilder};
use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};

pub fn open_app() -> eframe::Result<(), Error> {
    let viewport = ViewportBuilder::default();
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
        Ok(Box::new(Player))
    }))
}

struct Player;

impl App for Player {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        
    }
}


