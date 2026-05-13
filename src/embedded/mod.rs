
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum AudioSample {
    Roni,
    Json,
    Freddy,
}

impl AudioSample {
    pub fn wav(&self) -> &'static [u8] {
        match self {
            Self::Roni => include_bytes!("roni_sample.wav"),
            Self::Json => include_bytes!("json_sample.wav"),
            Self::Freddy => include_bytes!("freddy_sample.wav"),
        }
    }
    
    pub fn txt(&self) -> &'static str {
        match self {
            Self::Roni => include_str!("roni_sample.txt"),
            Self::Json => include_str!("json_sample.txt"),
            Self::Freddy => include_str!("freddy_sample.txt"),
        }
    }
}
