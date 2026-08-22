use crate::formats;
use std::fmt;

pub enum Mapping {
    AllFromSource,
    MainVideoAudioOnly,
}

impl Mapping {
    pub fn append_args(&self, args: &mut Vec<String>) {
        let slice: &[String] = match self {
            Self::AllFromSource => &formats!["-map", "0"],
            Self::MainVideoAudioOnly => &formats!["-map", "0:v:0?", "-map", "0:a:0?"],
        };
        args.extend_from_slice(slice);
    }
}

impl fmt::Display for Mapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = Vec::new();
        self.append_args(&mut args);
        write!(f, "{}", args.join(" "))
    }
}
