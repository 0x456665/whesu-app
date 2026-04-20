use crate::models::{AttendanceRecord, ChildRecord, Gender, ParentInfo};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

const DEFAULT_DAILY_REPORT_DATE: (i32, u32, u32) = (2026, 4, 6);
const DEFAULT_WEEK_START_DATE: (i32, u32, u32) = (2026, 4, 6);

pub fn default_daily_report_date() -> NaiveDate {
    date(
        DEFAULT_DAILY_REPORT_DATE.0,
        DEFAULT_DAILY_REPORT_DATE.1,
        DEFAULT_DAILY_REPORT_DATE.2,
    )
}

pub fn default_week_start() -> NaiveDate {
    date(
        DEFAULT_WEEK_START_DATE.0,
        DEFAULT_WEEK_START_DATE.1,
        DEFAULT_WEEK_START_DATE.2,
    )
}

pub fn sample_children() -> Vec<ChildRecord> {
    vec![
        ChildRecord {
            id: 1,
            first_name: "Ava".to_string(),
            last_name: "Johnson".to_string(),
            date_of_birth: date(2020, 5, 14),
            gender: Gender::Female,
            parent: parent(
                "Monica",
                "Johnson",
                "123 Maple Street",
                "Plano",
                "TX",
                "75023",
                "972-555-0101",
                "monica.johnson@example.com",
            ),
        },
        ChildRecord {
            id: 2,
            first_name: "Noah".to_string(),
            last_name: "Carter".to_string(),
            date_of_birth: date(2019, 11, 3),
            gender: Gender::Male,
            parent: parent(
                "James",
                "Carter",
                "45 Birch Lane",
                "Frisco",
                "TX",
                "75035",
                "469-555-0112",
                "james.carter@example.com",
            ),
        },
        ChildRecord {
            id: 3,
            first_name: "Mia".to_string(),
            last_name: "Nguyen".to_string(),
            date_of_birth: date(2021, 2, 21),
            gender: Gender::Female,
            parent: parent(
                "Linh",
                "Nguyen",
                "780 Oak Avenue",
                "Allen",
                "TX",
                "75013",
                "214-555-0123",
                "linh.nguyen@example.com",
            ),
        },
        ChildRecord {
            id: 4,
            first_name: "Ethan".to_string(),
            last_name: "Brooks".to_string(),
            date_of_birth: date(2020, 8, 9),
            gender: Gender::Male,
            parent: parent(
                "Keisha",
                "Brooks",
                "98 Willow Drive",
                "McKinney",
                "TX",
                "75070",
                "972-555-0134",
                "keisha.brooks@example.com",
            ),
        },
        ChildRecord {
            id: 5,
            first_name: "Skylar".to_string(),
            last_name: "Reed".to_string(),
            date_of_birth: date(2019, 7, 28),
            gender: Gender::NonBinary,
            parent: parent(
                "Jordan",
                "Reed",
                "611 Cedar Court",
                "Richardson",
                "TX",
                "75080",
                "214-555-0145",
                "jordan.reed@example.com",
            ),
        },
    ]
}

pub fn sample_attendance() -> Vec<AttendanceRecord> {
    let school_days = [
        date(2026, 3, 30),
        date(2026, 3, 31),
        date(2026, 4, 1),
        date(2026, 4, 2),
        date(2026, 4, 3),
        date(2026, 4, 6),
        date(2026, 4, 7),
        date(2026, 4, 8),
        date(2026, 4, 9),
        date(2026, 4, 10),
    ];

    let mut records = Vec::new();

    for (day_index, school_day) in school_days.iter().enumerate() {
        for child_id in 1..=5_u32 {
            // One split-day session demonstrates that reports handle multiple records per child.
            if child_id == 1 && *school_day == default_daily_report_date() {
                records.push(attendance_record(child_id, *school_day, (8, 0), (11, 45)));
                records.push(attendance_record(child_id, *school_day, (12, 30), (15, 15)));
                continue;
            }

            let start_hour = 7 + ((child_id + day_index as u32) % 3);
            let start_minute = if (child_id + day_index as u32) % 2 == 0 { 15 } else { 30 };
            let total_minutes = 435 + (day_index as i64 * 7) + (child_id as i64 * 9);

            let check_in = school_day.and_time(NaiveTime::from_hms_opt(start_hour, start_minute, 0).unwrap());
            let check_out = check_in + Duration::minutes(total_minutes);

            records.push(AttendanceRecord {
                child_id,
                check_in,
                check_out: Some(check_out),
            });
        }
    }

    records
}

fn parent(
    first_name: &str,
    last_name: &str,
    address: &str,
    city: &str,
    state: &str,
    zip_code: &str,
    phone_number: &str,
    email: &str,
) -> ParentInfo {
    ParentInfo {
        first_name: first_name.to_string(),
        last_name: last_name.to_string(),
        address: address.to_string(),
        city: city.to_string(),
        state: state.to_string(),
        zip_code: zip_code.to_string(),
        phone_number: phone_number.to_string(),
        email: email.to_string(),
    }
}

fn attendance_record(
    child_id: u32,
    attendance_date: NaiveDate,
    check_in: (u32, u32),
    check_out: (u32, u32),
) -> AttendanceRecord {
    AttendanceRecord {
        child_id,
        check_in: datetime(attendance_date, check_in.0, check_in.1),
        check_out: Some(datetime(attendance_date, check_out.0, check_out.1)),
    }
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn datetime(attendance_date: NaiveDate, hour: u32, minute: u32) -> NaiveDateTime {
    attendance_date.and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
}