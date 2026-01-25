use std::env::consts::OS;

use eframe::{App, Error, Frame, NativeOptions};
use egui::{Color32, Context, FontData, FontFamily, ViewportBuilder, epaint::text::{FontInsert, FontPriority, InsertFontFamily}};
use rfd::FileDialog;

enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

pub fn open_app() -> eframe::Result<(), Error> {
    let viewport = ViewportBuilder::default()
        .with_inner_size([1280.0, 720.0]);
    let options = NativeOptions {
        viewport,
        centered: true,
        persist_window: true,
        ..NativeOptions::default()
    };

    eframe::run_native("MIDI Player", options, Box::new(|cc| {
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

        let app = Box::new(Player {
            open_file_name: String::new(),
            state: PlayerState::Stopped,
            tick: 0,
            is_repeat: false,
            selection: Default::default(),
        });

        Ok(app)
    }))
}

struct Player {
    open_file_name: String,
    tick: u64,
    state: PlayerState,
    is_repeat: bool,
    selection: std::collections::HashSet<usize>,
}

impl Player {
    fn table_ui(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        // 테이블 빌더 설정
        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(
                Column::remainder()
                    .at_least(40.0)
                    .clip(true)
                    .resizable(true),
            )
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .sense(egui::Sense::click());

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Track");
                });
                header.col(|ui| {
                    ui.strong("Channel");
                });
                header.col(|ui| {
                    ui.strong("Note");
                });
            })
            .body(|mut body| {
                // 트랙 및 노트 테이블 표시 (더미)
                for row_index in 0..10 {
                    body.row(50.0, |mut row| {
                        row.set_selected(self.selection.contains(&row_index));
                        row.set_overline(true);

                        row.col(|ui| {
                            ui.label(row_index.to_string());
                            let mut checked = false;
                            ui.checkbox(&mut checked, "Mute");
                        });
                        row.col(|ui| {
                            ui.label((row_index + 1).to_string());
                        });
                        row.col(|ui| {
                            ui.add(egui::Separator::default().horizontal());
                        });

                        self.toggle_row_selection(row_index, &row.response());
                    });
                }
            });
    }

    fn toggle_row_selection(&mut self, row_index: usize, row_response: &egui::Response) {
        if row_response.clicked() {
            if self.selection.contains(&row_index) {
                self.selection.remove(&row_index);
            } else {
                self.selection.insert(row_index);
            }
        }
    }
}

impl App for Player {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let ctrl = if OS == "macos" { "⌘" } else { "Ctrl +" };

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let open_btn = egui::Button::new("Open").shortcut_text(format!("{} O", ctrl));
                    if ui.add(open_btn).clicked() {
                        // 파일 열기 대화상자 표시
                        if let Some(path) = FileDialog::default().pick_file() {
                            let file_name = path.file_name().unwrap();
                            self.open_file_name = String::from(file_name.to_str().unwrap());
                        }
                    }

                    let close_btn = egui::Button::new("Close").shortcut_text(format!("{} W", ctrl));
                    if ui.add(close_btn).clicked() {
                        // 파일 닫기
                        self.open_file_name.clear();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("control_panel").show(ctx, |ui| {
            let is_file_open = !self.open_file_name.is_empty();
            let title = if !is_file_open {
                "Select a MIDI file to play."
            } else {
                self.open_file_name.as_str()
            };
            ui.label(title);

            ui.horizontal_centered(|ui| {
                let rewind_btn = egui::Button::new("⏮");
                let is_not_begin = is_file_open && self.tick != 0;
                if ui.add_enabled(is_not_begin, rewind_btn).on_hover_text("Rewind").clicked() {
                    // 처음으로
                    self.tick = 0;
                }
                let is_playing = is_file_open && matches!(self.state, PlayerState::Playing);
                let play_pause_text = if is_playing { "⏸" } else { "▶" };
                let play_pause_btn = egui::Button::new(play_pause_text);
                if ui.add_enabled(is_file_open, play_pause_btn).on_hover_text("Play/Pause").clicked() {
                    // 재생/일시정지
                    self.state = match self.state {
                        PlayerState::Playing => PlayerState::Paused,
                        _ => PlayerState::Playing,
                    }
                }
                let stop_btn = egui::Button::new("⏹");
                if ui.add_enabled(is_playing, stop_btn).on_hover_text("Stop").clicked() {
                    // 정지
                    self.tick = 0;
                    self.state = PlayerState::Stopped;
                }
                let repeat_text = if self.is_repeat { "🔁" } else { "🔂" };
                let repeat_btn = egui::Button::new(repeat_text);
                if ui.add_enabled(is_file_open, repeat_btn).on_hover_text("Repeat").clicked() {
                    // 반복
                    self.is_repeat = !self.is_repeat;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let color = if ctx.theme().default_visuals().dark_mode { Color32::WHITE } else { Color32::BLACK };
            ui.visuals_mut().override_text_color = Some(color);
            
            // 트랙 및 노트 테이블
            use egui_extras::{Size, StripBuilder};
            StripBuilder::new(ui)
                .size(Size::remainder().at_least(100.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            self.table_ui(ui);
                        });
                    });
                });
        });
    }
}
