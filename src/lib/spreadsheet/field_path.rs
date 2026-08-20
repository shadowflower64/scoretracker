use std::fmt::Display;

/// A dot-separated field name.
///
/// This structure stores segments of a path for easier access.
///
/// # Examples
/// ```
/// # use scoretracker::spreadsheet::field_path::FieldPath;
/// assert_eq!(FieldPath::from("song_id"), FieldPath(vec!["song_id".to_string()]));
/// assert_eq!(FieldPath::from("chart.x.total_notes"), FieldPath(vec!["chart".to_string(), "x".to_string(), "total_notes".to_string()]));
/// assert_eq!(FieldPath(vec!["chart".to_string(), "x".to_string(), "total_notes".to_string()]).to_string().as_str(), "chart.x.total_notes");
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct FieldPath(pub Vec<String>);

impl<T: AsRef<str>> From<T> for FieldPath {
    fn from(value: T) -> Self {
        Self(value.as_ref().split(".").map(|x| x.to_owned()).collect())
    }
}

impl Display for FieldPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}
