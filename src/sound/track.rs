
pub struct Track {
    pub id: u8,
    pub channel: u8,
    pub instrument: u8,
    pub volume: f32,
    pub is_muted: bool,
    pub is_solo: bool,
}
