use crate::{
    models::{AttendanceRecord, ChildRecord, Gender, ParentInfo},
    sample_data,
};
use chrono::NaiveDateTime;
use dirs_next::{data_dir, data_local_dir};
use hex;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub struct DataStore {
    connection: Connection,
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

fn db_file_path() -> std::io::Result<PathBuf> {
    let base_dir = data_local_dir().or_else(data_dir).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not locate application data directory",
        )
    })?;

    let app_dir = base_dir.join("whesu_app");
    std::fs::create_dir_all(&app_dir)?;
    Ok(app_dir.join("daycare.db"))
}

impl DataStore {
    pub fn new_seeded() -> rusqlite::Result<Self> {
        let connection = Connection::open(db_file_path().map_err(|e| rusqlite::Error::InvalidPath(e.to_string().into()))?)?;
        let store = Self { connection };
        store.create_schema()?;
        store.migrate_schema()?;
        store.setup_default_password()?;
        store.seed_if_empty()?;
        Ok(store)
    }

    #[cfg(test)]
    pub fn new_for_test() -> rusqlite::Result<Self> {
        let connection = Connection::open_in_memory()?;
        let store = Self { connection };
        store.create_schema()?;
        store.setup_default_password()?;
        store.seed_if_empty()?;
        Ok(store)
    }

    // ── Password management ──────────────────────────────────────────────────

    pub fn verify_password(&self, input: &str) -> bool {
        let hash = hash_password(input);
        self.get_setting("password_hash")
            .ok()
            .flatten()
            .map(|stored| stored == hash)
            .unwrap_or(false)
    }

    pub fn update_password(&self, new_password: &str) -> rusqlite::Result<()> {
        let hash = hash_password(new_password);
        self.set_setting("password_hash", &hash)
    }

    fn get_setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        self.connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
    }

    fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    // ── Children ─────────────────────────────────────────────────────────────

    pub fn load_children(&self) -> rusqlite::Result<Vec<ChildRecord>> {
        let mut statement = self.connection.prepare(
            "
            SELECT
                id, first_name, last_name, date_of_birth, gender,
                parent_first_name, parent_last_name, address, city, state,
                zip_code, phone_number, email,
                allergies, emergency_contact_name, emergency_contact_phone
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
                allergies: row.get(13)?,
                emergency_contact_name: row.get(14)?,
                emergency_contact_phone: row.get(15)?,
            })
        })?;

        rows.collect()
    }

    pub fn add_child(&self, child: &ChildRecord) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            INSERT INTO children (
                id, first_name, last_name, date_of_birth, gender,
                parent_first_name, parent_last_name, address, city, state,
                zip_code, phone_number, email,
                allergies, emergency_contact_name, emergency_contact_phone
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
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
                child.allergies,
                child.emergency_contact_name,
                child.emergency_contact_phone,
            ],
        )?;
        Ok(())
    }

    pub fn update_child(&self, child: &ChildRecord) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            UPDATE children
            SET
                first_name = ?2,  last_name = ?3,  date_of_birth = ?4, gender = ?5,
                parent_first_name = ?6,  parent_last_name = ?7,
                address = ?8,  city = ?9,  state = ?10,  zip_code = ?11,
                phone_number = ?12,  email = ?13,
                allergies = ?14,  emergency_contact_name = ?15,  emergency_contact_phone = ?16
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
                child.allergies,
                child.emergency_contact_name,
                child.emergency_contact_phone,
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

    // ── Attendance ───────────────────────────────────────────────────────────

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

    pub fn check_out(
        &self,
        child_id: u32,
        check_out_time: NaiveDateTime,
    ) -> rusqlite::Result<i64> {
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

    // ── Schema ───────────────────────────────────────────────────────────────

    fn create_schema(&self) -> rusqlite::Result<()> {
        self.connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS children (
                id                      INTEGER PRIMARY KEY,
                first_name              TEXT NOT NULL,
                last_name               TEXT NOT NULL,
                date_of_birth           TEXT NOT NULL,
                gender                  TEXT NOT NULL,
                parent_first_name       TEXT NOT NULL,
                parent_last_name        TEXT NOT NULL,
                address                 TEXT NOT NULL,
                city                    TEXT NOT NULL,
                state                   TEXT NOT NULL,
                zip_code                TEXT NOT NULL,
                phone_number            TEXT NOT NULL,
                email                   TEXT NOT NULL,
                allergies               TEXT NOT NULL DEFAULT '',
                emergency_contact_name  TEXT NOT NULL DEFAULT '',
                emergency_contact_phone TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS attendance (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                child_id   INTEGER NOT NULL,
                check_in   TEXT NOT NULL,
                check_out  TEXT,
                FOREIGN KEY (child_id) REFERENCES children(id)
            );
            ",
        )
    }

    /// Add new columns to an existing database; errors are silently ignored
    /// when the column already exists (e.g., on a freshly-created database).
    fn migrate_schema(&self) -> rusqlite::Result<()> {
        let _ = self.connection.execute_batch(
            "ALTER TABLE children ADD COLUMN allergies TEXT NOT NULL DEFAULT ''",
        );
        let _ = self.connection.execute_batch(
            "ALTER TABLE children ADD COLUMN emergency_contact_name TEXT NOT NULL DEFAULT ''",
        );
        let _ = self.connection.execute_batch(
            "ALTER TABLE children ADD COLUMN emergency_contact_phone TEXT NOT NULL DEFAULT ''",
        );
        Ok(())
    }

    fn setup_default_password(&self) -> rusqlite::Result<()> {
        if self.get_setting("password_hash")?.is_none() {
            let hash = hash_password("password123");
            self.set_setting("password_hash", &hash)?;
        }
        Ok(())
    }

    fn seed_if_empty(&self) -> rusqlite::Result<()> {
        let row_count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM children", [], |row| row.get(0))?;

        if row_count > 0 {
            return Ok(());
        }

        for child in sample_data::sample_children() {
            self.connection.execute(
                "
                INSERT INTO children (
                    id, first_name, last_name, date_of_birth, gender,
                    parent_first_name, parent_last_name, address, city, state,
                    zip_code, phone_number, email,
                    allergies, emergency_contact_name, emergency_contact_phone
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
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
                    child.allergies,
                    child.emergency_contact_name,
                    child.emergency_contact_phone,
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
        let store = DataStore::new_for_test().unwrap();
        let children = store.load_children().unwrap();
        let attendance = store.load_attendance().unwrap();
        assert_eq!(children.len(), 5);
        assert!(attendance.len() >= 50);
    }

    #[test]
    fn default_password_is_password123() {
        let store = DataStore::new_for_test().unwrap();
        assert!(store.verify_password("password123"));
        assert!(!store.verify_password("wrong"));
    }

    #[test]
    fn can_change_password() {
        let store = DataStore::new_for_test().unwrap();
        store.update_password("newpass!").unwrap();
        assert!(!store.verify_password("password123"));
        assert!(store.verify_password("newpass!"));
    }

    #[test]
    fn inserts_a_child_record() {
        let store = DataStore::new_for_test().unwrap();
        let new_child = ChildRecord {
            id: 6,
            first_name: "Zoe".to_string(),
            last_name: "Parker".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(2021, 6, 12).unwrap(),
            gender: Gender::Female,
            parent: ParentInfo {
                first_name: "Ari".to_string(),
                last_name: "Parker".to_string(),
                address: "1 Demo Street".to_string(),
                city: "Dallas".to_string(),
                state: "TX".to_string(),
                zip_code: "75001".to_string(),
                phone_number: "+1(214)-555-0199".to_string(),
                email: "ari.parker@example.com".to_string(),
            },
            allergies: "None".to_string(),
            emergency_contact_name: "Max Parker".to_string(),
            emergency_contact_phone: "+1(214)-555-0200".to_string(),
        };
        store.add_child(&new_child).unwrap();
        let children = store.load_children().unwrap();
        assert_eq!(children.len(), 6);
        let zoe = children.iter().find(|c| c.id == 6).unwrap();
        assert_eq!(zoe.first_name, "Zoe");
        assert_eq!(zoe.allergies, "None");
        assert_eq!(zoe.emergency_contact_phone, "+1(214)-555-0200");
    }
}


