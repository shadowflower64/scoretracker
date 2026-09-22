use serde::{Deserialize, Serialize};

use crate::util::timestamp::NsTimestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guess<T> {
    prediction: T,
    confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction<T> {
    program_identifier: String,
    timestamp: NsTimestamp,
    guesses: Vec<Guess<T>>,
}

pub type AutomaticallyDetected<T> = Vec<Prediction<T>>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutomaticContentDetectionInformation {
    song_title: AutomaticallyDetected<String>,
    song_artist: AutomaticallyDetected<String>,
    song_id: AutomaticallyDetected<String>,
    player_name: AutomaticallyDetected<String>,
    instrument: AutomaticallyDetected<String>,
    difficulty: AutomaticallyDetected<String>,
    score: AutomaticallyDetected<f64>,
    note_streak: AutomaticallyDetected<u64>,
    note_hits: AutomaticallyDetected<u64>,
    notes_total: AutomaticallyDetected<u64>,
}
