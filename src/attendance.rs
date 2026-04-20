use crate::models::AttendanceRecord;
#[cfg(test)]
use chrono::NaiveDateTime;

pub fn is_checked_in(records: &[AttendanceRecord], child_id: u32) -> bool {
    records
        .iter()
        .any(|record| record.child_id == child_id && record.check_out.is_none())
}

#[cfg(test)]
pub fn check_in(
    records: &mut Vec<AttendanceRecord>,
    child_id: u32,
    check_in_time: NaiveDateTime,
) -> Result<(), String> {
    if is_checked_in(records, child_id) {
        return Err("This child is already checked in.".to_string());
    }

    records.push(AttendanceRecord {
        child_id,
        check_in: check_in_time,
        check_out: None,
    });

    Ok(())
}

#[cfg(test)]
pub fn check_out(
    records: &mut [AttendanceRecord],
    child_id: u32,
    check_out_time: NaiveDateTime,
) -> Result<i64, String> {
    let Some(open_record) = records
        .iter_mut()
        .rev()
        .find(|record| record.child_id == child_id && record.check_out.is_none())
    else {
        return Err("This child is not currently checked in.".to_string());
    };

    if check_out_time < open_record.check_in {
        return Err("Check-out time cannot be earlier than check-in time.".to_string());
    }

    open_record.check_out = Some(check_out_time);

    Ok(duration_minutes(open_record).unwrap_or_default())
}

pub fn duration_minutes(record: &AttendanceRecord) -> Option<i64> {
    record
        .check_out
        .map(|check_out| (check_out - record.check_in).num_minutes().max(0))
}

#[cfg(test)]
mod tests {
    use super::{check_in, check_out, duration_minutes, is_checked_in};
    use crate::models::AttendanceRecord;
    use chrono::{NaiveDate, NaiveTime};

    fn timestamp(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(hour, minute, 0).unwrap())
    }

    #[test]
    fn calculates_duration_in_minutes() {
        let record = AttendanceRecord {
            child_id: 1,
            check_in: timestamp(2026, 4, 6, 8, 0),
            check_out: Some(timestamp(2026, 4, 6, 15, 30)),
        };

        assert_eq!(duration_minutes(&record), Some(450));
    }

    #[test]
    fn prevents_duplicate_check_in() {
        let mut records = vec![AttendanceRecord {
            child_id: 1,
            check_in: timestamp(2026, 4, 6, 8, 0),
            check_out: None,
        }];

        let result = check_in(&mut records, 1, timestamp(2026, 4, 6, 8, 5));

        assert!(result.is_err());
        assert!(is_checked_in(&records, 1));
    }

    #[test]
    fn checks_out_open_record() {
        let mut records = vec![AttendanceRecord {
            child_id: 3,
            check_in: timestamp(2026, 4, 7, 8, 10),
            check_out: None,
        }];

        let result = check_out(&mut records, 3, timestamp(2026, 4, 7, 16, 10));

        assert_eq!(result, Ok(480));
        assert!(!is_checked_in(&records, 3));
    }
}