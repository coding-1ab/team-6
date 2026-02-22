use std::collections::HashMap;

use crate::audio::note::MidiNote;

// 실제 오디오 처리 로직을 담은 '내부 상태' 구조체
pub struct SharedAudioState {
    pub is_playing: bool,
    pub is_repeat: bool,
    pub playback_cursor: usize,
    pub notes: Vec<MidiNote>,
    pub programs: HashMap<u8, u8>,
}

impl SharedAudioState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            is_repeat: false,
            playback_cursor: 0,
            notes: vec![],
            programs: HashMap::new(),
        }
    }
}
