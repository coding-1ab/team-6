pub mod midi_instruments;

use std::env::consts::OS;
use eframe::{App, Error, Frame, NativeOptions};
use egui::{
    Color32,
    Context,
    FontData,
    FontFamily,
    Pos2,
    Slider,
    SliderOrientation,
    Stroke,
    ViewportBuilder,
    epaint::text::{FontInsert, FontPriority, InsertFontFamily}
};
use rfd::FileDialog;

use crate::{sound::track::Track, ui::midi_instruments::{DRUM_KITS, MIDI_GROUPS, MIDI_INSTRUMENTS}};

// 전체 UI 레이아웃 예시
// +-------------------------------------------------------------+
// | [1. TOP] Menu                                               |
// +-------------+---------------------------------+-------------+
// |             | [3. CENTER - TOP] Transport     |             |
// |             +---------------------------------+             |
// | [2. LEFT]   |                                 | [5. RIGHT]  |
// |  Track List | [4. CENTER - MAIN]              |  Inspector  |
// |             |  Piano Roll / Grid              |             |
// |             |                                 |             |
// |-------------+---------------------------------+-------------+
// | [6. BOTTOM]                                                 |
// |  Velocity / Controller                                      |
// +-------------------------------------------------------------+

const NOTE_NAMES: &[&'static str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
];

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

        // 더미 트랙 데이터
        let traks: Vec<Track> = vec![
            Track {
                id: 1,
                channel: 1,
                instrument: 1,
                volume: 80.0,
                is_muted: false,
                is_solo: false,
            },
            Track {
                id: 2,
                channel: 2,
                instrument: 22,
                volume: 100.0,
                is_muted: true,
                is_solo: false,
            },
            Track {
                id: 3,
                channel: 3,
                instrument: 36,
                volume: 80.0,
                is_muted: false,
                is_solo: false,
            },
        ];

        let app = Box::new(Player {
            open_file_name: String::new(),
            state: PlayerState::Stopped,
            tick: 0,
            is_repeat: false,
            tracks: traks,
            selected_track_id: None,
            solo_track_id: None,
        });

        Ok(app)
    }))
}

struct Player {
    open_file_name: String,
    tick: u64,
    state: PlayerState,
    is_repeat: bool,
    tracks: Vec<Track>,
    selected_track_id: Option<u8>,
    solo_track_id: Option<u8>,
}

impl Player {
    fn set_menu_bar(&mut self, ui: &mut egui::Ui) {
        // 1. TOP : 메뉴 예시
        // +-------------------------------------------------------------+
        // | File Edit View Help                                         |
        // +-------------------------------------------------------------+
        let ctrl = if OS == "macos" { "⌘" } else { "Ctrl +" };

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
    }

    fn set_track_list(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        ui.heading("Track List".to_string());
        ui.separator();

        // 테이블 빌더 설정
        let available_height = ui.available_height();
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::exact(210.0))
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(20.0, |mut header| {
                // 헤더 설정
                header.col(|ui| {
                    let text_height = ui.text_style_height(&egui::TextStyle::Body);
                    ui.horizontal(|ui| {
                        ui.add_sized([128.0, text_height], egui::Label::new("Track"));
                        ui.add_sized([18.0, text_height], egui::Label::new("CH"));
                        ui.add_sized([17.0, text_height], egui::Label::new("S"));
                        ui.add_sized([18.0, text_height], egui::Label::new("M"));
                    });
                });
            })
            .body(|mut body| self.set_track(&mut body));
    }

    fn set_track(&mut self, body: &mut egui_extras::TableBody<'_>) {
        for track in self.tracks.iter_mut() {
            let is_selected = self.selected_track_id == Some(track.id);
            let row_height = if is_selected { 96.0 } else { 20.0 };
            let instrument = MIDI_INSTRUMENTS.get(track.instrument as usize)
                .unwrap();
            let drum = match DRUM_KITS.get(&track.instrument) {
                Some(value) => value,
                None => DRUM_KITS.get(&0u8).unwrap(),
            };

            body.row(row_height, |mut row| {
                row.col(|ui| {
                    let label_width = 52.0;
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);

                    ui.vertical(|ui| {
                        // 2-1. LEFT : 트랙 리스트 (축소) 예시
                        // +------------------+----+------+------+--------------+
                        // | Track            | CH | Solo | Mute | Volume       |
                        // +------------------+----+------+------+--------------+
                        // | ▶ 1 Piano Melody | 01 | [ ]  | [x]  | ------O- 80% |
                        // | ▶ 2 Piano Melody | 01 | [x]  | [ ]  | -------O100% |
                        // | ▶ 3 Piano Melody | 01 | [ ]  | [ ]  | ------O- 80% |
                        // |                         ...                        |
                        // +----------------------------------------------------+
                        ui.horizontal(|ui| {
                            // 트랙 정보 자세히 보기 버튼
                            let btn_text = if is_selected { "-" } else { "+" };
                            let accordion_btn = egui::Button::new(btn_text).small();
                            let accordion_btn_ui = ui.add_sized([20.0, row_height], accordion_btn)
                                .on_hover_text("Details");
                            if accordion_btn_ui.clicked() {
                                self.selected_track_id = if !is_selected { Some(track.id) } else { None };
                            }

                            // 트랙 정보
                            let instrument_name = if track.channel != 10 { instrument.name } else { drum };
                            let track_label = egui::Label::new(format!("{} {}", track.instrument, instrument_name));
                            ui.add_sized([100.0, row_height], track_label);

                            // 채널 정보
                            let channel_label = egui::Label::new(format!("{:0>2}", track.channel));
                            ui.add_sized([20.0, row_height], channel_label);

                            // 솔로 체크박스
                            let mut is_solo = self.solo_track_id == Some(track.id);
                            if ui.checkbox(&mut is_solo, "").clicked() {
                                self.solo_track_id = if is_solo { Some(track.id) } else { None };
                            }

                            // 뮤트 체크박스
                            ui.checkbox(&mut track.is_muted, "");
                        });

                        if !is_selected { return; }
                        // 선택한 행의 경우 아래 확장 내용을 표시함

                        // 2-2. LEFT : 트랙 리스트 (확장) 예시
                        // | ▶ 1 Piano Melody | 01 | [ ]  | [x]  | ------O- 80% |
                        // +------------------+----+------+------+--------------+
                        // |  Solo: [ ]     Mute: [x]    Volume: ------O- 80%   |
                        // |  CH: [ 01 ▼ ]  Instrument: [ Piano Melody ]        |
                        // +------------------+----+------+------+--------------+
                        // | ▶ 2 Piano Melody | 01 | [x]  | [ ]  | -------O100% |

                        ui.separator();

                        // channel, track name
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            // 채널 드롭박스
                            ui.add_sized([label_width, row_height], egui::Label::new("Channel:"));
                            egui::ComboBox::from_id_salt("channel")
                                .width(40.0)
                                .selected_text(format!("{:0>2}", track.channel))
                                .show_ui(ui, |ui| {
                                    for ch in 0..=16 {
                                        ui.selectable_value(&mut track.channel, ch, format!("{:0>2}", ch));
                                    }
                                });
                        });

                        // instrument
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            // 악기 드롭박스
                            ui.add_sized([label_width, row_height], egui::Label::new("Instrument:"));
                            let mut selected_text = instrument.name;
                            let mut instruments: Vec<(u8, String)> = Vec::new();
                            if track.channel != 10 {
                                // 채널 10이 아닐 경우 일반 악기
                                for (idx, instrument) in MIDI_INSTRUMENTS.iter().enumerate() {
                                    let group = MIDI_GROUPS.get(instrument.group as usize).unwrap();
                                    let select_option_name = format!("{}. [{}] {}", idx, group.name, instrument.name);
                                    instruments.push((idx as u8, select_option_name));
                                }
                            } else {
                                // 채널 10일 경우 타악기
                                selected_text = drum;
                                for (idx, instrument) in DRUM_KITS.into_iter() {
                                    let select_option_name = format!("{}. {}", idx, instrument);
                                    instruments.push((*idx as u8, select_option_name));
                                }
                                instruments.sort();
                            }
                            egui::ComboBox::from_id_salt("instrument")
                                .width(140.0)
                                .selected_text(format!("{}", selected_text))
                                .show_ui(ui, |ui| {
                                    for (idx, instrument) in instruments {
                                        ui.selectable_value(&mut track.instrument, idx as u8, instrument);
                                    }
                                });
                        });

                        // volume
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            // 볼륨 슬라이더
                            ui.add_sized([label_width, row_height], egui::Label::new("Volume:"));
                            ui.horizontal(|ui| {
                                let slider = egui::Slider::new(&mut track.volume, 0.0..=100.0)
                                    .show_value(false);
                                ui.add_enabled(!track.is_muted, slider);

                                // 볼륨 퍼센테이지
                                let font_id = egui::FontId::new(12.0, egui::FontFamily::default());
                                let volume_text = format!("{}%", track.volume as u8);
                                let volume_pos = ui.min_rect().center();
                                ui.painter().text(
                                    volume_pos, egui::Align2::CENTER_CENTER,
                                    volume_text, font_id, Color32::GRAY,
                                );
                            });
                        });

                        ui.separator();
                    });
                });
            });
        }
    }

    fn set_transport(&mut self, ui: &mut egui::Ui) {
        // 3. CENTER - TOP : 트랜스포트 (MIDI 재생 제어) 예시
        // +-------------------------------------------------------------+
        // | midi_file.mid                                               |
        // | [⏮] [⏸] [⏹] [🔁]   ------O---------                        |
        // +-------------------------------------------------------------+
        let is_file_open = !self.open_file_name.is_empty();
        ui.horizontal_centered(|ui| {
            ui.vertical(|ui| {
                let title = if !is_file_open {
                    // 파일이 열려있지 않을 때 MIDI 파일을 선택해달라는 메시지 표시
                    "Select a MIDI file to play."
                } else {
                    // 파일이 열려있을 때 파일 이름 표시
                    self.open_file_name.as_str()
                };
                ui.add_sized([200.0, 22.0], egui::Label::new(title));

                // 컨트롤러 버튼
                ui.horizontal_centered(|ui| {
                    // 파일 열기
                    let is_opend = !self.open_file_name.is_empty();
                    let open_btn = egui::Button::new(if is_opend { "Close" } else { "Open" });
                    let open_btn_ui = ui.add(open_btn);
                    if open_btn_ui.clicked() {
                        if is_opend {
                            self.open_file_name.clear();
                        } else {
                            if let Some(path) = FileDialog::default().pick_file() {
                                let file_name = path.file_name().unwrap();
                                self.open_file_name = String::from(file_name.to_str().unwrap());
                            }
                        }
                    }

                    // 처음으로
                    let rewind_btn = egui::Button::new("⏮");
                    let is_not_begin = is_file_open && self.tick != 0;
                    let rewind_btn_ui = ui.add_enabled(is_not_begin, rewind_btn)
                        .on_hover_text("Rewind");
                    if rewind_btn_ui.clicked() {
                        self.tick = 0;
                    }

                    // 재생/일시정지
                    let is_playing = is_file_open && matches!(self.state, PlayerState::Playing);
                    let play_pause_text = if is_playing { "⏸" } else { "▶" };
                    let play_pause_btn = egui::Button::new(play_pause_text);
                    let play_pause_btn_ui = ui.add_enabled(is_file_open, play_pause_btn)
                        .on_hover_text("Play/Pause");
                    if play_pause_btn_ui.clicked() {
                        self.state = match self.state {
                            PlayerState::Playing => PlayerState::Paused,
                            _ => PlayerState::Playing,
                        }
                    }

                    // 정지
                    let stop_btn = egui::Button::new("⏹");
                    let stop_btn_ui = ui.add_enabled(is_playing, stop_btn)
                        .on_hover_text("Stop");
                    if stop_btn_ui.clicked() {
                        self.tick = 0;
                        self.state = PlayerState::Stopped;
                    }

                    // 반복
                    let repeat_text = if self.is_repeat { "🔁" } else { "🔂" };
                    let repeat_btn = egui::Button::new(repeat_text);
                    let repeat_btn_ui = ui.add_enabled(is_file_open, repeat_btn)
                        .on_hover_text("Repeat");
                    if repeat_btn_ui.clicked() {
                        self.is_repeat = !self.is_repeat;
                    }

                    // 재생 슬라이더
                    let mut value_i32 = 0;
                    ui.add(
                        Slider::new(&mut value_i32, 0..=100)
                            .orientation(SliderOrientation::Horizontal)
                            .step_by(1.0)
                            .trailing_fill(true)
                            .show_value(false),
                    );
                });
            });
        });
    }

    fn set_grid(&mut self, ui: &mut egui::Ui) {
        // 4. CENTER - MAIN : 피아노 롤 / 그리드 예시
        // +-----+-------+-------+-------+-------+-------+-------+-------+
        // | G9  |       | [===] |[===]  |       |       |       |       |
        // | F#9 |       |       |       |       |       |       |       |
        // | F9  |       |       |     [===========]     |       |       |
        // | E9  |       |    [=====]    |       |   [==]|       |       |
        // | D#9 |       |       |       |       |       |       |       |
        // |                              ...                            |
        // | D3  |       |       |       | [===] |       |       |       |
        // +-----+-------+-------+-------+-------+-------+-------+-------+
        let max_width = ui.available_width();
        let max_height = 128.0 * 12.0;
        ui.vertical(|ui| {
            let painter = ui.painter();
            painter.line_segment(
                [Pos2::new(250.0, 60.0), Pos2::new(max_width, 60.0)],
                Stroke::new(11.0, Color32::LIGHT_GRAY),
            );
        });

        egui::ScrollArea::both()
            .max_height(max_height)
            .hscroll(true)
            .vscroll(true)
            .auto_shrink(false)
            .show(ui, |ui| {
                let size = egui::vec2(max_width, max_height);
                let (response, _rect) = ui.allocate_exact_size(size, egui::Sense::hover());
                let painter = ui.painter().with_clip_rect(ui.clip_rect());
                let font_id = egui::FontId::new(10.0, egui::FontFamily::default());

                // 기준 좌표 계산
                let start_x = response.min.x;
                let start_y = response.min.y;
                let label_width = 35.0; // 음 이름 표시 너비 값
                let grid_start_x = start_x + label_width;
                let width = response.max.x;
                let height = response.max.y;

                for i in 0..=127 {
                    let note = 127 - i;
                    let color = match note % 12 {
                        1 | 3 | 6 | 8 | 10 => Color32::GRAY,
                        _ => Color32::LIGHT_GRAY,
                    };
                    let y = start_y + (i * 12) as f32;

                    // 음 이름 표시 영역 배경색
                    let rgb = color.r() - 50;
                    painter.rect_filled(
                        egui::Rect::from_points(&[
                            Pos2::new(start_x, y + 1.0),
                            Pos2::new(start_x + label_width - 1.0, y + 12.0)
                        ]),
                        0.0, Color32::from_rgb(rgb, rgb, rgb),
                    );

                    // 음 이름 텍스트 표시
                    painter.text(
                        Pos2::new(start_x + 3.0, y),
                        egui::Align2::LEFT_TOP,
                        format!("{}{}", NOTE_NAMES[note % 12], (note / 12) as i8),
                        font_id.clone(),
                        Color32::from_rgb(34, 34, 34),
                    );
                    
                    // 노트 영역 배경색
                    let start_point = Pos2::new(grid_start_x, y + 6.0);
                    let end_point = Pos2::new(width, y + 6.0);
                    painter.line_segment(
                        [start_point, end_point],
                        Stroke::new(11.0, color),
                    );
                }

                // 박자 구분을 위한 가이드 라인
                for i in 1..=(width/50.0) as usize {
                    let x = grid_start_x + (i * 50) as f32;
                    let start_point = Pos2::new(x, start_y);
                    let end_point = Pos2::new(x, height);
                    painter.line_segment(
                        [start_point, end_point],
                        Stroke::new(1.0, Color32::DARK_GRAY),
                    );
                }
            });
    }

    fn set_inspector(&mut self, ui: &mut egui::Ui) {
        // 5. RIGHT : 인스펙터 (선택된 노트/이벤트 속성) 예시
        // +-----------------------------+
        // | Note Properties             |
        // +-----------------------------+
        // | Pitch: 60 (C4)              |
        // | Velocity: 100               |
        // | Duration: 480 ticks         |
        // | Start Time: 960 ticks       |
        // |             ...             |
        // +-----------------------------+
        ui.heading("Note Properties".to_string());
        ui.separator();

        let ui_builder = egui::UiBuilder::new();
        ui.scope_builder(ui_builder, |ui| {
            egui::Grid::new("inspector")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.add_sized([80.0, 22.0], egui::Label::new("Name"));
                    ui.add_sized([110.0, 22.0], egui::Label::new("Value"));
                    ui.end_row();

                    let mut pitch = format!("{}", 60);
                    ui.add(egui::Label::new("Pitch"));
                    let edit_box = egui::TextEdit::singleline(&mut pitch)
                        .desired_width(40.0);
                    if ui.add(edit_box).changed() {
                        // pitch;
                    }
                    ui.end_row();

                    ui.add(egui::Label::new("Velocity"));
                    let mut velocity = format!("{}", 100);
                    let edit_box = egui::TextEdit::singleline(&mut velocity)
                        .desired_width(40.0);
                    if ui.add(edit_box).changed() {
                        // velocity;
                    }
                    ui.end_row();

                    ui.add(egui::Label::new("Duration"));
                    let mut duration = format!("{}", 480);
                    ui.horizontal(|ui| {
                        let edit_box = egui::TextEdit::singleline(&mut duration)
                            .desired_width(40.0);
                        if ui.add(edit_box).changed() {
                            // duration;
                        }
                        ui.label("ticks");
                    });
                    ui.end_row();

                    ui.add(egui::Label::new("Start Time"));
                    let mut start_time = format!("{}", 960);
                    ui.horizontal(|ui| {
                        let edit_box = egui::TextEdit::singleline(&mut start_time)
                            .desired_width(40.0);
                        if ui.add(edit_box).changed() {
                            // start_time;
                        }
                        ui.label("ticks");
                    });
                    ui.end_row();
                });
        });
    }

    fn set_velocity(&mut self, ui: &mut egui::Ui) {
        // 6. BOTTOM : 벨로시티 (피아노 건반) 예시
        // +---------------------------------------------------------+
        // | Velocity                                                |
        // +---------------------------------------------------------+
        // | |=|=|||=|=|=|||=|=|||=|=|=|||=|=|||=|=|=|||=|=|||=|=|=| |
        // |  | | | | | | | | | | | | | | | | | | | | | | | | | | |  |
        // +---------------------------------------------------------+
        ui.heading("Velocity".to_string());
        ui.separator();

        ui.label("Velocity Panel (피아노 건반 벨로시티 표시 영역)");
    }
}

impl App for Player {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // TOP 메뉴 바
        egui::TopBottomPanel::top("menu_bar")
            .exact_height(24.0)
            .resizable(false)
            .show(ctx, |ui| self.set_menu_bar(ui));

        // BOTTOM : 벨로시티 (피아노 건반)
        egui::TopBottomPanel::bottom("velocity")
            .exact_height(200.0)
            .resizable(false)
            .show(ctx, |ui| self.set_velocity(ui));

        // LEFT : 트랙 리스트
        egui::SidePanel::left("track_list")
            .exact_width(225.0)
            .resizable(false)
            .show(ctx, |ui| self.set_track_list(ui));

        // RIGHT : 인스펙터
        egui::SidePanel::right("inspector")
            .exact_width(225.0)
            .resizable(false)
            .show(ctx, |ui| self.set_inspector(ui));

        // CENTER - TOP : 트랜스포트
        egui::TopBottomPanel::top("transport")
            .exact_height(52.0)
            .resizable(false)
            .show(ctx, |ui| self.set_transport(ui));

        // CENTER - MAIN : 피아노 롤 / 그리드
        egui::CentralPanel::default()
            .show(ctx, |ui| self.set_grid(ui));
    }
}
