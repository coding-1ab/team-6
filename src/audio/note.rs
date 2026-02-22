
#[derive(Clone, Debug)]
pub struct MidiMeta {
    pub track_number: u16,
    pub program_name: String,
    pub tempo: f64,
    pub time_signature: [u8;4],
    pub key_signature: i8,
    pub is_minor: bool,
    pub text: String,
    pub copyright: String,
    pub track_name: String,
    pub instrument_name: String,
    pub marker: String,
}

#[derive(Clone, Debug)]
pub struct MidiData {
    pub is_on: bool,
    pub tick: u64,
    pub velocity: u8,
}

#[derive(Clone, Debug)]
pub struct MidiNote {
    pub start_sample: usize,
    pub channel: i32,
    pub key: i32,
    pub velocity: i32,
}
