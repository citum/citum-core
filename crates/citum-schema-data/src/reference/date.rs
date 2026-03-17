use crate::reference::types::RefDate;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// An EDTF string.
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct EdtfString(pub String);

impl EdtfString {
    /// Parse the string as an EDTF date etc, or return the string as a literal.
    pub fn parse(&self) -> RefDate {
        let mut input = self.0.as_str();
        match citum_edtf::parse(&mut input) {
            Ok(edtf) => RefDate::Edtf(edtf),
            Err(_) => RefDate::Literal(self.0.clone()),
        }
    }

    /// Extract the year from the date.
    pub fn year(&self) -> String {
        match self.parse() {
            RefDate::Edtf(edtf) => edtf.year().to_string(),
            RefDate::Literal(_) => String::new(),
        }
    }

    /// Extract the day from the date.
    pub fn day(&self) -> Option<u32> {
        match self.parse() {
            RefDate::Edtf(edtf) => edtf.day().filter(|&d| d > 0),
            RefDate::Literal(_) => None,
        }
    }

    /// Check if the date is uncertain (has "?" qualifier).
    pub fn is_uncertain(&self) -> bool {
        self.0.contains('?')
    }

    /// Check if the date is approximate (has "~" qualifier).
    pub fn is_approximate(&self) -> bool {
        self.0.contains('~')
    }

    /// Check if the date is a range (interval).
    pub fn is_range(&self) -> bool {
        matches!(self.parse(), RefDate::Edtf(edtf) if edtf.is_range())
    }

    /// Check if the range is open-ended (ends with "..").
    pub fn is_open_range(&self) -> bool {
        matches!(self.parse(), RefDate::Edtf(edtf) if edtf.is_open_range())
    }

    /// Extract the time component from the date, if present.
    pub fn time(&self) -> Option<citum_edtf::Time> {
        match self.parse() {
            RefDate::Edtf(edtf) => edtf.time(),
            _ => None,
        }
    }

    /// Check if the date has a time component.
    pub fn has_time(&self) -> bool {
        self.time().is_some()
    }
}

impl fmt::Display for EdtfString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
