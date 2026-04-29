use crate::{
    attendance,
    models::{AttendanceRecord, ChildRecord},
};
use chrono::{Datelike, Duration, NaiveDate};
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportRow {
    pub child_id: u32,
    pub child_name: String,
    pub total_minutes: i64,
    pub session_count: usize,
    pub incomplete_sessions: usize,
}

pub fn daily_report(
    children: &[ChildRecord],
    records: &[AttendanceRecord],
    date: NaiveDate,
    child_filter: Option<u32>,
) -> Vec<ReportRow> {
    build_report(children, records, child_filter, |record| {
        record.attendance_date() == date
    })
}

pub fn weekly_report(
    children: &[ChildRecord],
    records: &[AttendanceRecord],
    week_start: NaiveDate,
    child_filter: Option<u32>,
) -> Vec<ReportRow> {
    let week_end = week_start + Duration::days(6);
    build_report(children, records, child_filter, |record| {
        let d = record.attendance_date();
        d >= week_start && d <= week_end
    })
}

pub fn monthly_report(
    children: &[ChildRecord],
    records: &[AttendanceRecord],
    year: i32,
    month: u32,
    child_filter: Option<u32>,
) -> Vec<ReportRow> {
    build_report(children, records, child_filter, |record| {
        let d = record.attendance_date();
        d.year() == year && d.month() == month
    })
}

pub fn format_minutes(minutes: i64) -> String {
    let hours = minutes / 60;
    let remaining_minutes = minutes % 60;
    format!("{hours}h {remaining_minutes}m")
}

pub fn export_csv(rows: &[ReportRow], path: &str) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "Child,Sessions,Total Minutes,Total Time,Open Sessions")?;
    for row in rows {
        writeln!(
            file,
            "{},{},{},{},{}",
            escape_csv(&row.child_name),
            row.session_count,
            row.total_minutes,
            escape_csv(&format_minutes(row.total_minutes)),
            row.incomplete_sessions,
        )?;
    }
    Ok(())
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn build_report<F>(
    children: &[ChildRecord],
    records: &[AttendanceRecord],
    child_filter: Option<u32>,
    predicate: F,
) -> Vec<ReportRow>
where
    F: Fn(&AttendanceRecord) -> bool,
{
    let mut sorted_children = children.to_vec();
    sorted_children.sort_by(|a, b| {
        a.last_name
            .cmp(&b.last_name)
            .then_with(|| a.first_name.cmp(&b.first_name))
    });

    sorted_children
        .into_iter()
        .filter(|child| child_filter.map_or(true, |id| child.id == id))
        .map(|child| {
            let matching: Vec<_> = records
                .iter()
                .filter(|r| r.child_id == child.id && predicate(r))
                .collect();

            let total_minutes = matching
                .iter()
                .filter_map(|r| attendance::duration_minutes(r))
                .sum();

            let incomplete_sessions = matching.iter().filter(|r| r.check_out.is_none()).count();

            ReportRow {
                child_id: child.id,
                child_name: child.full_name(),
                total_minutes,
                session_count: matching.len(),
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
            date_of_birth: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            gender: Gender::PreferNotToSay,
            parent: ParentInfo {
                first_name: "Parent".to_string(),
                last_name: last_name.to_string(),
                address: "1 Demo Street".to_string(),
                city: "Dallas".to_string(),
                state: "TX".to_string(),
                zip_code: "75001".to_string(),
                phone_number: "+1(555)-000-0100".to_string(),
                email: "parent@example.com".to_string(),
            },
            allergies: String::new(),
            emergency_contact_name: String::new(),
            emergency_contact_phone: String::new(),
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
            None,
        );

        let ava = report.iter().find(|r| r.child_name == "Ava Jones").unwrap();
        let liam = report.iter().find(|r| r.child_name == "Liam Smith").unwrap();
        assert_eq!(ava.total_minutes, 390);
        assert_eq!(ava.session_count, 2);
        assert_eq!(liam.total_minutes, 480);
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
            None,
        );

        let ava = report.iter().find(|r| r.child_name == "Ava Jones").unwrap();
        assert_eq!(ava.total_minutes, 1_260);
        assert_eq!(ava.session_count, 3);
    }

    #[test]
    fn child_filter_limits_results() {
        let children = vec![child(1, "Ava", "Jones"), child(2, "Liam", "Smith")];
        let records = vec![
            session(1, 2026, 4, 6, 8, 0, 16, 0),
            session(2, 2026, 4, 6, 8, 0, 16, 0),
        ];
        let report = daily_report(
            &children,
            &records,
            NaiveDate::from_ymd_opt(2026, 4, 6).unwrap(),
            Some(1),
        );
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].child_name, "Ava Jones");
    }
}

