use crate::{attendance, models::{AttendanceRecord, ChildRecord}};
use chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRow {
    pub child_name: String,
    pub total_minutes: i64,
    pub session_count: usize,
    pub incomplete_sessions: usize,
}

pub fn daily_report(
    children: &[ChildRecord],
    records: &[AttendanceRecord],
    date: NaiveDate,
) -> Vec<ReportRow> {
    build_report(children, records, |record| record.attendance_date() == date)
}

pub fn weekly_report(
    children: &[ChildRecord],
    records: &[AttendanceRecord],
    week_start: NaiveDate,
) -> Vec<ReportRow> {
    let week_end = week_start + Duration::days(6);
    build_report(children, records, |record| {
        let attendance_date = record.attendance_date();
        attendance_date >= week_start && attendance_date <= week_end
    })
}

pub fn format_minutes(minutes: i64) -> String {
    let hours = minutes / 60;
    let remaining_minutes = minutes % 60;
    format!("{minutes} minutes ({hours}h {remaining_minutes}m)")
}

fn build_report<F>(
    children: &[ChildRecord],
    records: &[AttendanceRecord],
    predicate: F,
) -> Vec<ReportRow>
where
    F: Fn(&AttendanceRecord) -> bool,
{
    let mut sorted_children = children.to_vec();
    sorted_children.sort_by(|left, right| {
        left.last_name
            .cmp(&right.last_name)
            .then_with(|| left.first_name.cmp(&right.first_name))
    });

    sorted_children
        .into_iter()
        .map(|child| {
            let matching_records: Vec<_> = records
                .iter()
                .filter(|record| record.child_id == child.id && predicate(record))
                .collect();

            let total_minutes = matching_records
                .iter()
                .filter_map(|record| attendance::duration_minutes(record))
                .sum();

            let incomplete_sessions = matching_records
                .iter()
                .filter(|record| record.check_out.is_none())
                .count();

            ReportRow {
                child_name: child.full_name(),
                total_minutes,
                session_count: matching_records.len(),
                incomplete_sessions,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{daily_report, weekly_report};
    use crate::models::{AttendanceRecord, ChildRecord, Gender, ParentInfo};
    use chrono::{NaiveDate, NaiveTime};

    fn child(id: u32, first_name: &str, last_name: &str) -> ChildRecord {
        ChildRecord {
            id,
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(2020, 1, id).unwrap(),
            gender: Gender::PreferNotToSay,
            parent: ParentInfo {
                first_name: "Parent".to_string(),
                last_name: last_name.to_string(),
                address: "1 Demo Street".to_string(),
                city: "Dallas".to_string(),
                state: "TX".to_string(),
                zip_code: "75001".to_string(),
                phone_number: "555-0100".to_string(),
                email: "parent@example.com".to_string(),
            },
        }
    }

    fn session(
        child_id: u32,
        year: i32,
        month: u32,
        day: u32,
        in_hour: u32,
        in_minute: u32,
        out_hour: u32,
        out_minute: u32,
    ) -> AttendanceRecord {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let check_in = date.and_time(NaiveTime::from_hms_opt(in_hour, in_minute, 0).unwrap());
        let check_out = date.and_time(NaiveTime::from_hms_opt(out_hour, out_minute, 0).unwrap());

        AttendanceRecord {
            child_id,
            check_in,
            check_out: Some(check_out),
        }
    }

    #[test]
    fn aggregates_daily_totals_for_multiple_sessions() {
        let children = vec![child(1, "Ava", "Jones"), child(2, "Liam", "Smith")];
        let records = vec![
            session(1, 2026, 4, 6, 8, 0, 11, 0),
            session(1, 2026, 4, 6, 12, 0, 15, 30),
            session(2, 2026, 4, 6, 8, 15, 16, 15),
        ];

        let report = daily_report(
            &children,
            &records,
            NaiveDate::from_ymd_opt(2026, 4, 6).unwrap(),
        );

        let ava_row = report.iter().find(|row| row.child_name == "Ava Jones").unwrap();
        let liam_row = report.iter().find(|row| row.child_name == "Liam Smith").unwrap();

        assert_eq!(ava_row.total_minutes, 390);
        assert_eq!(ava_row.session_count, 2);
        assert_eq!(liam_row.total_minutes, 480);
        assert_eq!(liam_row.session_count, 1);
    }

    #[test]
    fn aggregates_weekly_totals_across_days() {
        let children = vec![child(1, "Ava", "Jones"), child(2, "Liam", "Smith")];
        let records = vec![
            session(1, 2026, 4, 6, 8, 0, 16, 0),
            session(1, 2026, 4, 7, 8, 0, 15, 0),
            session(1, 2026, 4, 10, 8, 0, 14, 0),
            session(2, 2026, 4, 8, 8, 30, 16, 0),
        ];

        let report = weekly_report(
            &children,
            &records,
            NaiveDate::from_ymd_opt(2026, 4, 6).unwrap(),
        );

        let ava_row = report.iter().find(|row| row.child_name == "Ava Jones").unwrap();
        let liam_row = report.iter().find(|row| row.child_name == "Liam Smith").unwrap();

        assert_eq!(ava_row.total_minutes, 1_260);
        assert_eq!(ava_row.session_count, 3);
        assert_eq!(liam_row.total_minutes, 450);
    }
}