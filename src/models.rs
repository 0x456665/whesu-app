use chrono::{NaiveDate, NaiveDateTime};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Login,
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Female,
    Male,
    NonBinary,
    PreferNotToSay,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Female => "Female",
            Self::Male => "Male",
            Self::NonBinary => "Non-binary",
            Self::PreferNotToSay => "Prefer not to say",
        };

        f.write_str(label)
    }
}

impl Gender {
    pub fn from_storage(value: &str) -> Self {
        match value {
            "Female" => Self::Female,
            "Male" => Self::Male,
            "Non-binary" => Self::NonBinary,
            _ => Self::PreferNotToSay,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentInfo {
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub phone_number: String,
    pub email: String,
}

impl ParentInfo {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildRecord {
    pub id: u32,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: NaiveDate,
    pub gender: Gender,
    pub parent: ParentInfo,
    pub allergies: String,
    pub emergency_contact_name: String,
    pub emergency_contact_phone: String,
}

impl ChildRecord {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceRecord {
    pub child_id: u32,
    pub check_in: NaiveDateTime,
    pub check_out: Option<NaiveDateTime>,
}

impl AttendanceRecord {
    pub fn attendance_date(&self) -> NaiveDate {
        self.check_in.date()
    }
}