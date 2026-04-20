# Happy Haven Day Care Center

This project is a Rust desktop application built with iced for Windows-friendly daycare attendance tracking. It uses bundled SQLite in memory so the app still runs as a single binary with no external database file, no internet access, and embedded sample data.

## Planned structure

- `src/models.rs`: strongly typed child, parent, gender, and attendance record models.
- `src/sample_data.rs`: embedded child records plus at least 10 school days of attendance history for demonstrations.
- `src/storage.rs`: bundled SQLite schema, demo-data seeding, and persistence helpers.
- `src/attendance.rs`: check-in, check-out, open-session validation, and duration calculations.
- `src/reports.rs`: daily and weekly aggregation logic plus formatting helpers for totals.
- `src/app.rs`: iced application state, message handling, tab navigation, and GUI layout.
- `src/main.rs`: application bootstrap and window configuration.

## Data model summary

- `ChildRecord` stores child name, date of birth, gender, and a nested `ParentInfo` record.
- `ParentInfo` stores parent name, address, city, state, zip code, phone number, and email address.
- `AttendanceRecord` stores `child_id`, `check_in`, `check_out`, and supports duration calculation through the attendance module.
- `DataStore` initializes an in-memory SQLite database, creates schema, and seeds the embedded sample records on startup.

## Features

- Add child records with parent contact details
- Edit existing child and parent records
- Delete child records and their related attendance history
- Separate Check In and Check Out attendance views
- Daily attendance report with total hours and minutes per child
- Weekly attendance report with total hours and minutes per child
- Embedded sample child records and 10 days of attendance history
- Bundled SQLite running entirely in memory for a single-binary Windows deliverable

## Using the app

### Add a child record

1. Open the `Children` tab.
2. Complete the `Add Child` form.
3. Select `Add Child`.
4. The new child is saved immediately and the app switches to `Attendance` so you can record the first check-in.

### Edit a child record

1. Open the `Children` tab.
2. Find the child in the roster table.
3. Select `Edit` on that row.
4. Update the form fields.
5. Select `Save Changes`.

### Delete a child record

1. Open the `Children` tab.
2. Find the child in the roster table.
3. Select `Delete`.
4. The child record and related attendance sessions are removed from the in-memory database.

### Check in a child

1. Open the `Attendance` tab.
2. Stay on the `Check In` sub-tab.
3. Select a child from the picker or use the `Check In` action in the table.
4. The child moves to the `Check Out` list after a successful check-in.

### Check out a child

1. Open the `Attendance` tab.
2. Switch to the `Check Out` sub-tab.
3. Select a child from the picker or use the `Check Out` action in the table.
4. The session duration is saved and the child moves back to the `Check In` list.

## Build and run

Run the app locally:

```bash
cargo run
```

Run tests:

```bash
cargo test
```

Create a Windows release build:

```bash
cargo build --release --target x86_64-pc-windows-msvc
```

If you prefer the GNU target for a simpler one-file demo setup and have the toolchain installed:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The resulting executable will be placed under the matching target directory, for example:

```text
target/x86_64-pc-windows-msvc/release/whesu_app.exe
```

Because SQLite is compiled into the application using rusqlite's `bundled` feature, the program does not need a separate SQLite installation or `.db` file for the demo.

## Demo notes

- The default daily report date is `2026-04-06`.
- The default weekly report start date is `2026-04-06`.
- Sample data includes a split attendance day for one child so daily and weekly reports demonstrate multiple sessions correctly.