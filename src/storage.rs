use crate::{models::{AttendanceRecord, ChildRecord, Gender, ParentInfo}, sample_data};
use chrono::NaiveDateTime;
use rusqlite::{params, Connection, OptionalExtension};

pub struct DataStore {
    connection: Connection,
}

impl DataStore {
    pub fn new_seeded() -> rusqlite::Result<Self> {
        let connection = Connection::open_in_memory()?;
        let store = Self { connection };
        store.create_schema()?;
        store.seed_if_empty()?;
        Ok(store)
    }

    pub fn load_children(&self) -> rusqlite::Result<Vec<ChildRecord>> {
        let mut statement = self.connection.prepare(
            "
            SELECT
                id,
                first_name,
                last_name,
                date_of_birth,
                gender,
                parent_first_name,
                parent_last_name,
                address,
                city,
                state,
                zip_code,
                phone_number,
                email
            FROM children
            ORDER BY last_name, first_name
            ",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(ChildRecord {
                id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                date_of_birth: row.get(3)?,
                gender: Gender::from_storage(&row.get::<_, String>(4)?),
                parent: ParentInfo {
                    first_name: row.get(5)?,
                    last_name: row.get(6)?,
                    address: row.get(7)?,
                    city: row.get(8)?,
                    state: row.get(9)?,
                    zip_code: row.get(10)?,
                    phone_number: row.get(11)?,
                    email: row.get(12)?,
                },
            })
        })?;

        rows.collect()
    }

    pub fn load_attendance(&self) -> rusqlite::Result<Vec<AttendanceRecord>> {
        let mut statement = self.connection.prepare(
            "
            SELECT child_id, check_in, check_out
            FROM attendance
            ORDER BY check_in DESC
            ",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(AttendanceRecord {
                child_id: row.get(0)?,
                check_in: row.get(1)?,
                check_out: row.get(2)?,
            })
        })?;

        rows.collect()
    }

    pub fn check_in(&self, child_id: u32, check_in_time: NaiveDateTime) -> rusqlite::Result<()> {
        let is_open: Option<i64> = self
            .connection
            .query_row(
                "SELECT id FROM attendance WHERE child_id = ?1 AND check_out IS NULL LIMIT 1",
                params![child_id],
                |row| row.get(0),
            )
            .optional()?;

        if is_open.is_some() {
            return Err(rusqlite::Error::InvalidQuery);
        }

        self.connection.execute(
            "INSERT INTO attendance (child_id, check_in, check_out) VALUES (?1, ?2, NULL)",
            params![child_id, check_in_time],
        )?;

        Ok(())
    }

    pub fn check_out(&self, child_id: u32, check_out_time: NaiveDateTime) -> rusqlite::Result<i64> {
        let open_session: Option<(i64, NaiveDateTime)> = self
            .connection
            .query_row(
                "
                SELECT id, check_in
                FROM attendance
                WHERE child_id = ?1 AND check_out IS NULL
                ORDER BY check_in DESC
                LIMIT 1
                ",
                params![child_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let Some((attendance_id, check_in_time)) = open_session else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        };

        let total_minutes = (check_out_time - check_in_time).num_minutes();
        if total_minutes < 0 {
            return Err(rusqlite::Error::InvalidQuery);
        }

        self.connection.execute(
            "UPDATE attendance SET check_out = ?1 WHERE id = ?2",
            params![check_out_time, attendance_id],
        )?;

        Ok(total_minutes)
    }

    pub fn add_child(&self, child: &ChildRecord) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            INSERT INTO children (
                id,
                first_name,
                last_name,
                date_of_birth,
                gender,
                parent_first_name,
                parent_last_name,
                address,
                city,
                state,
                zip_code,
                phone_number,
                email
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ",
            params![
                child.id,
                child.first_name,
                child.last_name,
                child.date_of_birth,
                child.gender.to_string(),
                child.parent.first_name,
                child.parent.last_name,
                child.parent.address,
                child.parent.city,
                child.parent.state,
                child.parent.zip_code,
                child.parent.phone_number,
                child.parent.email,
            ],
        )?;

        Ok(())
    }

    pub fn update_child(&self, child: &ChildRecord) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            UPDATE children
            SET
                first_name = ?2,
                last_name = ?3,
                date_of_birth = ?4,
                gender = ?5,
                parent_first_name = ?6,
                parent_last_name = ?7,
                address = ?8,
                city = ?9,
                state = ?10,
                zip_code = ?11,
                phone_number = ?12,
                email = ?13
            WHERE id = ?1
            ",
            params![
                child.id,
                child.first_name,
                child.last_name,
                child.date_of_birth,
                child.gender.to_string(),
                child.parent.first_name,
                child.parent.last_name,
                child.parent.address,
                child.parent.city,
                child.parent.state,
                child.parent.zip_code,
                child.parent.phone_number,
                child.parent.email,
            ],
        )?;

        Ok(())
    }

    pub fn delete_child(&self, child_id: u32) -> rusqlite::Result<()> {
        self.connection
            .execute("DELETE FROM attendance WHERE child_id = ?1", params![child_id])?;
        self.connection
            .execute("DELETE FROM children WHERE id = ?1", params![child_id])?;

        Ok(())
    }

    fn create_schema(&self) -> rusqlite::Result<()> {
        // The schema stays in memory, but bundled SQLite lets the final Windows build remain a single executable.
        self.connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS children (
                id INTEGER PRIMARY KEY,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                date_of_birth TEXT NOT NULL,
                gender TEXT NOT NULL,
                parent_first_name TEXT NOT NULL,
                parent_last_name TEXT NOT NULL,
                address TEXT NOT NULL,
                city TEXT NOT NULL,
                state TEXT NOT NULL,
                zip_code TEXT NOT NULL,
                phone_number TEXT NOT NULL,
                email TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS attendance (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                child_id INTEGER NOT NULL,
                check_in TEXT NOT NULL,
                check_out TEXT,
                FOREIGN KEY (child_id) REFERENCES children(id)
            );
            ",
        )
    }

    fn seed_if_empty(&self) -> rusqlite::Result<()> {
        let row_count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM children", [], |row| row.get(0))?;

        if row_count > 0 {
            return Ok(());
        }

        // Seed the demo dataset directly into SQLite so the UI and reports can be explained against a real schema.
        for child in sample_data::sample_children() {
            self.connection.execute(
                "
                INSERT INTO children (
                    id,
                    first_name,
                    last_name,
                    date_of_birth,
                    gender,
                    parent_first_name,
                    parent_last_name,
                    address,
                    city,
                    state,
                    zip_code,
                    phone_number,
                    email
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ",
                params![
                    child.id,
                    child.first_name,
                    child.last_name,
                    child.date_of_birth,
                    child.gender.to_string(),
                    child.parent.first_name,
                    child.parent.last_name,
                    child.parent.address,
                    child.parent.city,
                    child.parent.state,
                    child.parent.zip_code,
                    child.parent.phone_number,
                    child.parent.email,
                ],
            )?;
        }

        for record in sample_data::sample_attendance() {
            self.connection.execute(
                "INSERT INTO attendance (child_id, check_in, check_out) VALUES (?1, ?2, ?3)",
                params![record.child_id, record.check_in, record.check_out],
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DataStore;
    use crate::models::{ChildRecord, Gender, ParentInfo};
    use chrono::NaiveDate;

    #[test]
    fn seeds_children_and_attendance() {
        let store = DataStore::new_seeded().unwrap();
        let children = store.load_children().unwrap();
        let attendance = store.load_attendance().unwrap();

        assert_eq!(children.len(), 5);
        assert!(attendance.len() >= 50);
    }

    #[test]
    fn inserts_a_child_record() {
        let store = DataStore::new_seeded().unwrap();
        let new_child = ChildRecord {
            id: 6,
            first_name: "Zoe".to_string(),
            last_name: "Parker".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(2021, 6, 12).unwrap(),
            gender: Gender::Female,
            parent: ParentInfo {
                first_name: "Ari".to_string(),
                last_name: "Parker".to_string(),
                address: "10 Pine Street".to_string(),
                city: "Plano".to_string(),
                state: "TX".to_string(),
                zip_code: "75024".to_string(),
                phone_number: "972-555-0188".to_string(),
                email: "ari.parker@example.com".to_string(),
            },
        };

        store.add_child(&new_child).unwrap();

        let children = store.load_children().unwrap();
        assert_eq!(children.len(), 6);
        assert!(children.iter().any(|child| child.full_name() == "Zoe Parker"));
    }

    #[test]
    fn updates_a_child_record() {
        let store = DataStore::new_seeded().unwrap();
        let mut child = store.load_children().unwrap().into_iter().find(|child| child.id == 1).unwrap();

        child.first_name = "Avery".to_string();
        child.parent.phone_number = "972-555-0199".to_string();

        store.update_child(&child).unwrap();

        let updated_child = store.load_children().unwrap().into_iter().find(|child| child.id == 1).unwrap();
        assert_eq!(updated_child.first_name, "Avery");
        assert_eq!(updated_child.parent.phone_number, "972-555-0199");
    }

    #[test]
    fn deletes_child_and_attendance_records() {
        let store = DataStore::new_seeded().unwrap();

        store.delete_child(1).unwrap();

        let children = store.load_children().unwrap();
        let attendance = store.load_attendance().unwrap();

        assert!(!children.iter().any(|child| child.id == 1));
        assert!(!attendance.iter().any(|record| record.child_id == 1));
    }
}