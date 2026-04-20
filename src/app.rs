use crate::{
    attendance,
    models::{AttendanceRecord, ChildRecord, Gender, ParentInfo},
    reports::{self, ReportRow},
    sample_data,
    storage::DataStore,
};
use chrono::{Local, NaiveDate};
use iced::{
    application,
    widget::{self, button, container, pick_list, scrollable, text, text_input, Column, Row},
    Element, Fill, Length, Size, Task, Theme,
};
use std::{cmp::Reverse, fmt};

const GENDERS: [Gender; 4] = [
    Gender::Female,
    Gender::Male,
    Gender::NonBinary,
    Gender::PreferNotToSay,
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChildOption {
    id: u32,
    label: String,
}

impl fmt::Display for ChildOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Children,
    Attendance,
    Reports,
}

impl Tab {
    const ALL: [Self; 3] = [Self::Children, Self::Attendance, Self::Reports];

    fn label(self) -> &'static str {
        match self {
            Self::Children => "Children",
            Self::Attendance => "Attendance",
            Self::Reports => "Reports",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttendanceTab {
    CheckIn,
    CheckOut,
}

impl AttendanceTab {
    const ALL: [Self; 2] = [Self::CheckIn, Self::CheckOut];

    fn label(self) -> &'static str {
        match self {
            Self::CheckIn => "Check In",
            Self::CheckOut => "Check Out",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportTab {
    Daily,
    Weekly,
}

impl ReportTab {
    const ALL: [Self; 2] = [Self::Daily, Self::Weekly];

    fn label(self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum StatusKind {
    Success,
    Error,
}

impl StatusKind {
    fn label(self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Error => "Error",
        }
    }
}

#[derive(Debug, Clone)]
struct StatusMessage {
    kind: StatusKind,
    text: String,
}

impl StatusMessage {
    fn new(kind: StatusKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ChildFormField {
    FirstName,
    LastName,
    DateOfBirth,
    ParentFirstName,
    ParentLastName,
    PhoneNumber,
    Email,
    Address,
    City,
    State,
    ZipCode,
}

#[derive(Debug, Clone)]
struct ChildForm {
    first_name: String,
    last_name: String,
    date_of_birth: String,
    gender: Gender,
    parent_first_name: String,
    parent_last_name: String,
    phone_number: String,
    email: String,
    address: String,
    city: String,
    state: String,
    zip_code: String,
}

impl Default for ChildForm {
    fn default() -> Self {
        Self {
            first_name: String::new(),
            last_name: String::new(),
            date_of_birth: String::new(),
            gender: Gender::PreferNotToSay,
            parent_first_name: String::new(),
            parent_last_name: String::new(),
            phone_number: String::new(),
            email: String::new(),
            address: String::new(),
            city: String::new(),
            state: "TX".to_string(),
            zip_code: String::new(),
        }
    }
}

impl ChildForm {
    fn set(&mut self, field: ChildFormField, value: String) {
        match field {
            ChildFormField::FirstName => self.first_name = value,
            ChildFormField::LastName => self.last_name = value,
            ChildFormField::DateOfBirth => self.date_of_birth = value,
            ChildFormField::ParentFirstName => self.parent_first_name = value,
            ChildFormField::ParentLastName => self.parent_last_name = value,
            ChildFormField::PhoneNumber => self.phone_number = value,
            ChildFormField::Email => self.email = value,
            ChildFormField::Address => self.address = value,
            ChildFormField::City => self.city = value,
            ChildFormField::State => self.state = value,
            ChildFormField::ZipCode => self.zip_code = value,
        }
    }

    fn build_child(&self, next_id: u32) -> Result<ChildRecord, String> {
        let first_name = required_value(&self.first_name, "Enter the child's first name.")?;
        let last_name = required_value(&self.last_name, "Enter the child's last name.")?;
        let parent_first_name = required_value(
            &self.parent_first_name,
            "Enter the parent or guardian first name.",
        )?;
        let parent_last_name = required_value(
            &self.parent_last_name,
            "Enter the parent or guardian last name.",
        )?;
        let phone_number = required_value(&self.phone_number, "Enter a phone number.")?;
        let date_of_birth = parse_date_value(
            &self.date_of_birth,
            "Enter the date of birth as YYYY-MM-DD.",
        )?;

        Ok(ChildRecord {
            id: next_id,
            first_name,
            last_name,
            date_of_birth,
            gender: self.gender,
            parent: ParentInfo {
                first_name: parent_first_name,
                last_name: parent_last_name,
                address: self.address.trim().to_string(),
                city: self.city.trim().to_string(),
                state: self.state.trim().to_string(),
                zip_code: self.zip_code.trim().to_string(),
                phone_number,
                email: self.email.trim().to_string(),
            },
        })
    }

    fn from_child(child: &ChildRecord) -> Self {
        Self {
            first_name: child.first_name.clone(),
            last_name: child.last_name.clone(),
            date_of_birth: child.date_of_birth.format("%Y-%m-%d").to_string(),
            gender: child.gender,
            parent_first_name: child.parent.first_name.clone(),
            parent_last_name: child.parent.last_name.clone(),
            phone_number: child.parent.phone_number.clone(),
            email: child.parent.email.clone(),
            address: child.parent.address.clone(),
            city: child.parent.city.clone(),
            state: child.parent.state.clone(),
            zip_code: child.parent.zip_code.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildFormMode {
    Add,
    Edit(u32),
}

impl ChildFormMode {
    fn title(self) -> &'static str {
        match self {
            Self::Add => "Add Child",
            Self::Edit(_) => "Edit Child",
        }
    }

    fn submit_label(self) -> &'static str {
        match self {
            Self::Add => "Add Child",
            Self::Edit(_) => "Save Changes",
        }
    }
}

pub(crate) struct DaycareApp {
    store: DataStore,
    children: Vec<ChildRecord>,
    attendance_records: Vec<AttendanceRecord>,
    active_tab: Tab,
    attendance_tab: AttendanceTab,
    report_tab: ReportTab,
    selected_child_id: Option<u32>,
    child_form_mode: ChildFormMode,
    child_form: ChildForm,
    daily_report_input: String,
    weekly_report_input: String,
    status: Option<StatusMessage>,
}

impl Default for DaycareApp {
    fn default() -> Self {
        let store = DataStore::new_seeded().expect("failed to initialize embedded SQLite datastore");
        let children = store.load_children().expect("failed to load child records");
        let attendance_records = store
            .load_attendance()
            .unwrap_or_else(|_| sample_data::sample_attendance());

        Self {
            store,
            children,
            attendance_records,
            active_tab: Tab::Children,
            attendance_tab: AttendanceTab::CheckIn,
            report_tab: ReportTab::Daily,
            selected_child_id: None,
            child_form_mode: ChildFormMode::Add,
            child_form: ChildForm::default(),
            daily_report_input: sample_data::default_daily_report_date()
                .format("%Y-%m-%d")
                .to_string(),
            weekly_report_input: sample_data::default_week_start().format("%Y-%m-%d").to_string(),
            status: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    TabSelected(Tab),
    AttendanceTabSelected(AttendanceTab),
    ReportTabSelected(ReportTab),
    ChildSelected(u32),
    OpenAttendanceForChild(u32),
    EditChild(u32),
    DeleteChild(u32),
    ResetChildForm,
    CheckInSelectedChild,
    CheckOutSelectedChild,
    CheckInChild(u32),
    CheckOutChild(u32),
    DailyReportInputChanged(String),
    WeeklyReportInputChanged(String),
    ChildFormChanged(ChildFormField, String),
    ChildGenderSelected(Gender),
    SaveChild,
}

pub(crate) fn run() -> iced::Result {
    application("Happy Haven Daycare", update, view)
        .theme(|_| Theme::Light)
        .window_size(Size::new(1280.0, 840.0))
        .run()
}

fn update(state: &mut DaycareApp, message: Message) -> Task<Message> {
    match message {
        Message::TabSelected(tab) => {
            state.active_tab = tab;
        }
        Message::AttendanceTabSelected(tab) => {
            state.attendance_tab = tab;
        }
        Message::ReportTabSelected(tab) => {
            state.report_tab = tab;
        }
        Message::ChildSelected(child_id) => {
            state.selected_child_id = Some(child_id);
        }
        Message::OpenAttendanceForChild(child_id) => {
            state.selected_child_id = Some(child_id);
            state.active_tab = Tab::Attendance;
            state.attendance_tab = if attendance::is_checked_in(&state.attendance_records, child_id) {
                AttendanceTab::CheckOut
            } else {
                AttendanceTab::CheckIn
            };
        }
        Message::EditChild(child_id) => state.start_editing_child(child_id),
        Message::DeleteChild(child_id) => state.delete_child(child_id),
        Message::ResetChildForm => state.reset_child_form(),
        Message::CheckInSelectedChild => match state.selected_child_id {
            Some(child_id) => state.check_in_child(child_id),
            None => state.set_status(StatusKind::Error, "Select a child to check in."),
        },
        Message::CheckOutSelectedChild => match state.selected_child_id {
            Some(child_id) => state.check_out_child(child_id),
            None => state.set_status(StatusKind::Error, "Select a child to check out."),
        },
        Message::CheckInChild(child_id) => state.check_in_child(child_id),
        Message::CheckOutChild(child_id) => state.check_out_child(child_id),
        Message::DailyReportInputChanged(value) => {
            state.daily_report_input = value;
        }
        Message::WeeklyReportInputChanged(value) => {
            state.weekly_report_input = value;
        }
        Message::ChildFormChanged(field, value) => {
            state.child_form.set(field, value);
        }
        Message::ChildGenderSelected(gender) => {
            state.child_form.gender = gender;
        }
        Message::SaveChild => {
            state.save_child();
        }
    }

    Task::none()
}

fn view(state: &DaycareApp) -> Element<'_, Message> {
    let mut layout = Column::new().spacing(18).padding(24);
    layout = layout.push(text("Happy Haven Daycare").size(34));
    layout = layout.push(view_overview(state));

    if let Some(status) = &state.status {
        layout = layout.push(view_status(status));
    }

    layout = layout.push(view_tabs(state.active_tab));
    layout = layout.push(match state.active_tab {
        Tab::Children => view_children_tab(state),
        Tab::Attendance => view_attendance_tab(state),
        Tab::Reports => view_reports_tab(state),
    });

    container(scrollable(layout).height(Fill))
        .width(Fill)
        .height(Fill)
        .into()
}

impl DaycareApp {
    fn reset_child_form(&mut self) {
        self.child_form_mode = ChildFormMode::Add;
        self.child_form = ChildForm::default();
    }

    fn reload_children(&mut self) {
        if let Ok(children) = self.store.load_children() {
            self.children = children;
        }

        if self.selected_child_id.is_none() {
            return;
        }

        let selected_is_valid = self
            .selected_child_id
            .is_some_and(|child_id| self.children.iter().any(|child| child.id == child_id));

        if !selected_is_valid {
            self.selected_child_id = self.children.first().map(|child| child.id);
        }
    }

    fn reload_attendance(&mut self) {
        if let Ok(records) = self.store.load_attendance() {
            self.attendance_records = records;
        }
    }

    fn child_label(&self, child_id: u32) -> String {
        self.children
            .iter()
            .find(|child| child.id == child_id)
            .map(|child| child.full_name())
            .unwrap_or_else(|| format!("Child #{child_id}"))
    }

    fn selected_child(&self) -> Option<&ChildRecord> {
        self.selected_child_id
            .and_then(|child_id| self.children.iter().find(|child| child.id == child_id))
    }

    fn set_status(&mut self, kind: StatusKind, message: impl Into<String>) {
        self.status = Some(StatusMessage::new(kind, message));
    }

    fn recent_records_for_child(&self, child_id: u32) -> Vec<&AttendanceRecord> {
        let mut records: Vec<_> = self
            .attendance_records
            .iter()
            .filter(|record| record.child_id == child_id)
            .collect();

        records.sort_by_key(|record| Reverse(record.check_in));
        records.truncate(6);
        records
    }

    fn child_options(&self) -> Vec<ChildOption> {
        self.children
            .iter()
            .map(|child| ChildOption {
                id: child.id,
                label: child.full_name(),
            })
            .collect()
    }

    fn selected_child_option(&self) -> Option<ChildOption> {
        self.selected_child_id.map(|child_id| ChildOption {
            id: child_id,
            label: self.child_label(child_id),
        })
    }

    fn next_child_id(&self) -> u32 {
        self.children.iter().map(|child| child.id).max().unwrap_or(0) + 1
    }

    fn currently_checked_in_count(&self) -> usize {
        self.children
            .iter()
            .filter(|child| attendance::is_checked_in(&self.attendance_records, child.id))
            .count()
    }

    fn status_label(&self, child_id: u32) -> &'static str {
        if attendance::is_checked_in(&self.attendance_records, child_id) {
            "Checked In"
        } else {
            "Checked Out"
        }
    }

    fn latest_record_for_child(&self, child_id: u32) -> Option<&AttendanceRecord> {
        self.attendance_records
            .iter()
            .filter(|record| record.child_id == child_id)
            .max_by_key(|record| record.check_in)
    }

    fn last_activity_label(&self, child_id: u32) -> String {
        match self.latest_record_for_child(child_id) {
            Some(record) => match record.check_out {
                Some(check_out) => format!("Out {}", check_out.format("%Y-%m-%d %I:%M %p")),
                None => format!("In {}", record.check_in.format("%Y-%m-%d %I:%M %p")),
            },
            None => "No sessions".to_string(),
        }
    }

    fn start_editing_child(&mut self, child_id: u32) {
        if let Some(child) = self.children.iter().find(|child| child.id == child_id) {
            self.selected_child_id = Some(child_id);
            self.child_form_mode = ChildFormMode::Edit(child_id);
            self.child_form = ChildForm::from_child(child);
            self.active_tab = Tab::Children;
        }
    }

    fn delete_child(&mut self, child_id: u32) {
        match self.store.delete_child(child_id) {
            Ok(()) => {
                let child_name = self.child_label(child_id);
                self.reload_children();
                self.reload_attendance();

                if self.selected_child_id == Some(child_id) {
                    self.selected_child_id = None;
                }

                if self.child_form_mode == ChildFormMode::Edit(child_id) {
                    self.reset_child_form();
                }

                self.set_status(StatusKind::Success, format!("Deleted {child_name}."));
            }
            Err(error) => {
                self.set_status(StatusKind::Error, format!("Could not delete child: {error}"));
            }
        }
    }

    fn check_in_child(&mut self, child_id: u32) {
        let timestamp = Local::now().naive_local();

        match self.store.check_in(child_id, timestamp) {
            Ok(()) => {
                self.selected_child_id = Some(child_id);
                self.reload_attendance();
                self.attendance_tab = AttendanceTab::CheckOut;
                self.set_status(
                    StatusKind::Success,
                    format!(
                        "Checked in {} at {}.",
                        self.child_label(child_id),
                        timestamp.format("%I:%M %p")
                    ),
                );
            }
            Err(_) => self.set_status(StatusKind::Error, "This child is already checked in."),
        }
    }

    fn check_out_child(&mut self, child_id: u32) {
        let timestamp = Local::now().naive_local();

        match self.store.check_out(child_id, timestamp) {
            Ok(total_minutes) => {
                self.selected_child_id = Some(child_id);
                self.reload_attendance();
                self.attendance_tab = AttendanceTab::CheckIn;
                self.set_status(
                    StatusKind::Success,
                    format!(
                        "Checked out {}. Session total: {}.",
                        self.child_label(child_id),
                        reports::format_minutes(total_minutes)
                    ),
                );
            }
            Err(_) => self.set_status(StatusKind::Error, "This child is not currently checked in."),
        }
    }

    fn save_child(&mut self) {
        let child_id = match self.child_form_mode {
            ChildFormMode::Add => self.next_child_id(),
            ChildFormMode::Edit(child_id) => child_id,
        };

        match self.child_form.build_child(child_id) {
            Ok(child) => match self.child_form_mode {
                ChildFormMode::Add => match self.store.add_child(&child) {
                    Ok(()) => {
                        let child_name = child.full_name();
                        self.reload_children();
                        self.selected_child_id = Some(child.id);
                        self.active_tab = Tab::Attendance;
                        self.attendance_tab = AttendanceTab::CheckIn;
                        self.reset_child_form();
                        self.set_status(
                            StatusKind::Success,
                            format!("Added {child_name}. Ready for attendance."),
                        );
                    }
                    Err(error) => {
                        self.set_status(StatusKind::Error, format!("Could not add child: {error}"));
                    }
                },
                ChildFormMode::Edit(_) => match self.store.update_child(&child) {
                    Ok(()) => {
                        let child_name = child.full_name();
                        self.reload_children();
                        self.selected_child_id = Some(child.id);
                        self.reset_child_form();
                        self.set_status(
                            StatusKind::Success,
                            format!("Updated {child_name}."),
                        );
                    }
                    Err(error) => {
                        self.set_status(StatusKind::Error, format!("Could not update child: {error}"));
                    }
                },
            },
            Err(error) => {
                self.set_status(StatusKind::Error, error);
            }
        }
    }
}

fn view_overview(state: &DaycareApp) -> Element<'_, Message> {
    let checked_in_now = state.currently_checked_in_count();
    let checked_out_now = state.children.len().saturating_sub(checked_in_now);

    Row::new()
        .spacing(12)
        .push(metric_card("Children", state.children.len().to_string()))
        .push(metric_card("Checked In", checked_in_now.to_string()))
        .push(metric_card("Checked Out", checked_out_now.to_string()))
        .into()
}

fn view_status(status: &StatusMessage) -> Element<'_, Message> {
    section_card(
        Column::new()
            .spacing(4)
            .push(text(status.kind.label()).size(16))
            .push(text(&status.text)),
    )
}

fn view_tabs(active_tab: Tab) -> Element<'static, Message> {
    let mut tabs = Row::new().spacing(10);

    for tab in Tab::ALL {
        let style = if tab == active_tab {
            widget::button::primary
        } else {
            widget::button::secondary
        };

        tabs = tabs.push(
            button(text(tab.label()))
                .width(Length::FillPortion(1))
                .padding([10, 16])
                .style(style)
                .on_press(Message::TabSelected(tab)),
        );
    }

    section_card(tabs)
}

fn view_children_tab(state: &DaycareApp) -> Element<'_, Message> {
    Column::new()
        .spacing(16)
        .push(section_card(view_add_child_form(state)))
        .push(section_card(view_children_table(state)))
        .into()
}

fn view_add_child_form(state: &DaycareApp) -> Element<'_, Message> {
    let gender_picker = pick_list(
        GENDERS,
        Some(state.child_form.gender),
        Message::ChildGenderSelected,
    )
    .placeholder("Select gender")
    .width(Fill);

    let mut actions = Row::new().spacing(12).push(
        button(text(state.child_form_mode.submit_label()))
            .padding([10, 18])
            .style(widget::button::primary)
            .on_press(Message::SaveChild),
    );

    if state.child_form_mode != ChildFormMode::Add {
        actions = actions.push(
            button(text("Cancel"))
                .padding([10, 18])
                .style(widget::button::secondary)
                .on_press(Message::ResetChildForm),
        );
    }

    Column::new()
        .spacing(12)
        .push(text(state.child_form_mode.title()).size(24))
        .push(
            Row::new()
                .spacing(12)
                .push(form_field(
                    "First name",
                    text_input("First name", &state.child_form.first_name)
                        .on_input(|value| Message::ChildFormChanged(ChildFormField::FirstName, value))
                        .padding(10),
                ))
                .push(form_field(
                    "Last name",
                    text_input("Last name", &state.child_form.last_name)
                        .on_input(|value| Message::ChildFormChanged(ChildFormField::LastName, value))
                        .padding(10),
                )),
        )
        .push(
            Row::new()
                .spacing(12)
                .push(form_field(
                    "Date of birth",
                    text_input("YYYY-MM-DD", &state.child_form.date_of_birth)
                        .on_input(|value| {
                            Message::ChildFormChanged(ChildFormField::DateOfBirth, value)
                        })
                        .padding(10),
                ))
                .push(form_field("Gender", gender_picker)),
        )
        .push(
            Row::new()
                .spacing(12)
                .push(form_field(
                    "Parent first name",
                    text_input("Parent first name", &state.child_form.parent_first_name)
                        .on_input(|value| {
                            Message::ChildFormChanged(ChildFormField::ParentFirstName, value)
                        })
                        .padding(10),
                ))
                .push(form_field(
                    "Parent last name",
                    text_input("Parent last name", &state.child_form.parent_last_name)
                        .on_input(|value| {
                            Message::ChildFormChanged(ChildFormField::ParentLastName, value)
                        })
                        .padding(10),
                )),
        )
        .push(
            Row::new()
                .spacing(12)
                .push(form_field(
                    "Phone",
                    text_input("Phone", &state.child_form.phone_number)
                        .on_input(|value| {
                            Message::ChildFormChanged(ChildFormField::PhoneNumber, value)
                        })
                        .padding(10),
                ))
                .push(form_field(
                    "Email",
                    text_input("Email", &state.child_form.email)
                        .on_input(|value| Message::ChildFormChanged(ChildFormField::Email, value))
                        .padding(10),
                )),
        )
        .push(form_field(
            "Address",
            text_input("Street address", &state.child_form.address)
                .on_input(|value| Message::ChildFormChanged(ChildFormField::Address, value))
                .padding(10),
        ))
        .push(
            Row::new()
                .spacing(12)
                .push(form_field(
                    "City",
                    text_input("City", &state.child_form.city)
                        .on_input(|value| Message::ChildFormChanged(ChildFormField::City, value))
                        .padding(10),
                ))
                .push(form_field(
                    "State",
                    text_input("State", &state.child_form.state)
                        .on_input(|value| Message::ChildFormChanged(ChildFormField::State, value))
                        .padding(10),
                ))
                .push(form_field(
                    "Zip code",
                    text_input("Zip code", &state.child_form.zip_code)
                        .on_input(|value| Message::ChildFormChanged(ChildFormField::ZipCode, value))
                        .padding(10),
                )),
        )
            .push(actions)
        .into()
}

fn view_children_table(state: &DaycareApp) -> Element<'_, Message> {
    let mut content = Column::new().spacing(10);
    content = content.push(text("Children").size(24).font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    }));
    content = content.push(children_table_header());

    if state.children.is_empty() {
        return content.push(empty_state("No children found. Expand the roster using the 'Add Child' tab.")).into();
    }

    for (i, child) in state.children.iter().enumerate() {
        let is_selected = state.selected_child_id == Some(child.id);
        let is_even = i % 2 == 0;
        
        let row = Row::new()
            .spacing(16)
            .push(table_cell(child.full_name(), 3))
            .push(table_cell(child.parent.full_name(), 3))
            .push(table_cell(child.date_of_birth.format("%Y-%m-%d").to_string(), 2))
            .push(table_cell(child.parent.phone_number.clone(), 2))
            .push(table_cell(state.status_label(child.id).to_string(), 2))
            .push(
                container(
                    Row::new()
                        .spacing(8)
                        .push(
                            button(text("Attend"))
                                .padding([8, 12])
                                .style(widget::button::secondary)
                                .on_press(Message::OpenAttendanceForChild(child.id)),
                        )
                        .push(
                            button(text("Edit"))
                                .padding([8, 12])
                                .style(widget::button::secondary)
                                .on_press(Message::EditChild(child.id)),
                        )
                        .push(
                            button(text("Delete"))
                                .padding([8, 12])
                                .style(widget::button::danger)
                                .on_press(Message::DeleteChild(child.id)),
                        ),
                )
                .width(Length::FillPortion(3)),
            );

        content = content.push(
            container(row)
                .padding([12, 16])
                .width(Fill)
                .style(move |theme: &Theme| table_row_style(theme, is_even, is_selected)),
        );
    }

    content.into()
}

fn view_attendance_tab(state: &DaycareApp) -> Element<'_, Message> {
    Column::new()
        .spacing(16)
        .push(section_card(view_attendance_tabs(state.attendance_tab)))
        .push(section_card(view_attendance_actions(state)))
        .push(section_card(view_selected_child_summary(state)))
        .push(section_card(match state.attendance_tab {
            AttendanceTab::CheckIn => view_check_in_table(state),
            AttendanceTab::CheckOut => view_check_out_table(state),
        }))
        .into()
}

fn view_attendance_tabs(active_tab: AttendanceTab) -> Element<'static, Message> {
    let mut tabs = Row::new().spacing(10);

    for tab in AttendanceTab::ALL {
        let style = if tab == active_tab {
            widget::button::primary
        } else {
            widget::button::secondary
        };

        tabs = tabs.push(
            button(text(tab.label()))
                .padding([8, 14])
                .style(style)
                .on_press(Message::AttendanceTabSelected(tab)),
        );
    }

    tabs.into()
}

fn view_attendance_actions(state: &DaycareApp) -> Element<'_, Message> {
    let child_picker = pick_list(
        state.child_options(),
        state.selected_child_option(),
        |child| Message::ChildSelected(child.id),
    )
    .placeholder("Select a child")
    .width(Fill);

    let action_button = match state.attendance_tab {
        AttendanceTab::CheckIn => {
            let enabled = state.selected_child_id.is_some_and(|child_id| {
                !attendance::is_checked_in(&state.attendance_records, child_id)
            });

            button(text("Check In"))
                .padding([10, 16])
                .style(widget::button::success)
                .on_press_maybe(enabled.then_some(Message::CheckInSelectedChild))
        }
        AttendanceTab::CheckOut => {
            let enabled = state.selected_child_id.is_some_and(|child_id| {
                attendance::is_checked_in(&state.attendance_records, child_id)
            });

            button(text("Check Out"))
                .padding([10, 16])
                .style(widget::button::danger)
                .on_press_maybe(enabled.then_some(Message::CheckOutSelectedChild))
        }
    };

    Column::new()
        .spacing(12)
        .push(text(state.attendance_tab.label()).size(24))
        .push(form_field("Selected child", child_picker))
        .push(action_button)
        .into()
}

fn view_selected_child_summary(state: &DaycareApp) -> Element<'_, Message> {
    let mut content = Column::new().spacing(10);
    content = content.push(text("Selected Child").size(24));

    if let Some(child) = state.selected_child() {
        content = content
            .push(
                Row::new()
                    .spacing(12)
                    .push(metric_card("Name", child.full_name()))
                    .push(metric_card("Status", state.status_label(child.id).to_string()))
                    .push(metric_card("Parent", child.parent.full_name())),
            )
            .push(text(format!("Phone: {}", child.parent.phone_number)));

        let recent_records = state.recent_records_for_child(child.id);
        if recent_records.is_empty() {
            content = content.push(empty_state("No attendance history yet."));
        } else {
            content = content.push(recent_sessions_header());

            for (i, record) in recent_records.into_iter().enumerate() {
                let is_even = i % 2 == 0;
                let check_out_label = record
                    .check_out
                    .map(|value| value.format("%I:%M %p").to_string())
                    .unwrap_or_else(|| "Open".to_string());
                let duration = attendance::duration_minutes(record)
                    .map(reports::format_minutes)
                    .unwrap_or_else(|| "Open".to_string());

                let row = Row::new()
                    .spacing(16)
                    .push(table_cell(record.check_in.date().format("%Y-%m-%d").to_string(), 2))
                    .push(table_cell(record.check_in.format("%I:%M %p").to_string(), 2))
                    .push(table_cell(check_out_label, 2))
                    .push(table_cell(duration, 3));

                content = content.push(
                    container(row)
                        .padding([12, 16])
                        .width(Fill)
                        .style(move |theme: &Theme| table_row_style(theme, is_even, false)),
                );
            }
        }
    } else {
        content = content.push(text("Select a child to review recent sessions."));
    }

    content.into()
}

fn view_check_in_table(state: &DaycareApp) -> Element<'_, Message> {
    let mut content = Column::new().spacing(10);
    let available_children: Vec<_> = state
        .children
        .iter()
        .filter(|child| !attendance::is_checked_in(&state.attendance_records, child.id))
        .collect();

    content = content.push(text("Ready to Check In").size(24).font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    }));

    if available_children.is_empty() {
        return content.push(empty_state("All children are already checked in.")).into();
    }

    content = content.push(attendance_table_header("Check In"));

    for (i, child) in available_children.iter().enumerate() {
        let is_selected = state.selected_child_id == Some(child.id);
        let is_even = i % 2 == 0;
        let action: Element<'_, Message> = button(text("Check In"))
            .padding([8, 16])
            .style(widget::button::success)
            .on_press(Message::CheckInChild(child.id))
            .into();

        let row = Row::new()
            .spacing(16)
            .push(table_cell(child.full_name(), 3))
            .push(table_cell(child.parent.full_name(), 3))
            .push(table_cell(state.status_label(child.id).to_string(), 2))
            .push(table_cell(state.last_activity_label(child.id), 3))
            .push(container(action).width(Length::FillPortion(2)));

        content = content.push(
            container(row)
                .padding([12, 16])
                .width(Fill)
                .style(move |theme: &Theme| table_row_style(theme, is_even, is_selected)),
        );
    }

    content.into()
}

fn view_check_out_table(state: &DaycareApp) -> Element<'_, Message> {
    let mut content = Column::new().spacing(10);
    let available_children: Vec<_> = state
        .children
        .iter()
        .filter(|child| attendance::is_checked_in(&state.attendance_records, child.id))
        .collect();

    content = content.push(text("Ready to Check Out").size(24).font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    }));

    if available_children.is_empty() {
        return content.push(empty_state("No children are currently checked in.")).into();
    }

    content = content.push(attendance_table_header("Check Out"));

    for (i, child) in available_children.iter().enumerate() {
        let is_selected = state.selected_child_id == Some(child.id);
        let is_even = i % 2 == 0;
        let action: Element<'_, Message> = button(text("Check Out"))
            .padding([8, 16])
            .style(widget::button::danger)
            .on_press(Message::CheckOutChild(child.id))
            .into();

        let row = Row::new()
            .spacing(16)
            .push(table_cell(child.full_name(), 3))
            .push(table_cell(child.parent.full_name(), 3))
            .push(table_cell(state.status_label(child.id).to_string(), 2))
            .push(table_cell(state.last_activity_label(child.id), 3))
            .push(container(action).width(Length::FillPortion(2)));

        content = content.push(
            container(row)
                .padding([12, 16])
                .width(Fill)
                .style(move |theme: &Theme| table_row_style(theme, is_even, is_selected)),
        );
    }

    content.into()
}

fn view_reports_tab(state: &DaycareApp) -> Element<'_, Message> {
    let mut content = Column::new().spacing(16);
    content = content.push(section_card(view_report_controls(state)));

    match state.report_tab {
        ReportTab::Daily => match parse_date_value(
            &state.daily_report_input,
            "Enter a valid date as YYYY-MM-DD.",
        ) {
            Ok(date) => {
                let rows = reports::daily_report(&state.children, &state.attendance_records, date);
                content = content.push(section_card(
                    Column::new()
                        .spacing(12)
                        .push(text(format!("Daily Report: {}", date.format("%A, %B %d, %Y"))).size(24))
                        .push(view_report_rows(&rows)),
                ));
            }
            Err(error) => {
                content = content.push(section_card(text(error)));
            }
        },
        ReportTab::Weekly => match parse_date_value(
            &state.weekly_report_input,
            "Enter a valid week start date as YYYY-MM-DD.",
        ) {
            Ok(week_start) => {
                let week_end = week_start + chrono::Duration::days(6);
                let rows = reports::weekly_report(&state.children, &state.attendance_records, week_start);
                content = content.push(section_card(
                    Column::new()
                        .spacing(12)
                        .push(text(format!(
                            "Weekly Report: {} to {}",
                            week_start.format("%Y-%m-%d"),
                            week_end.format("%Y-%m-%d")
                        ))
                        .size(24))
                        .push(view_report_rows(&rows)),
                ));
            }
            Err(error) => {
                content = content.push(section_card(text(error)));
            }
        },
    }

    content.into()
}

fn view_report_controls(state: &DaycareApp) -> Element<'_, Message> {
    let mut tabs = Row::new().spacing(10);
    for tab in ReportTab::ALL {
        let style = if tab == state.report_tab {
            widget::button::primary
        } else {
            widget::button::secondary
        };

        tabs = tabs.push(
            button(text(tab.label()))
                .padding([8, 14])
                .style(style)
                .on_press(Message::ReportTabSelected(tab)),
        );
    }

    let date_input: Element<'_, Message> = match state.report_tab {
        ReportTab::Daily => text_input("YYYY-MM-DD", &state.daily_report_input)
            .on_input(Message::DailyReportInputChanged)
            .padding(10)
            .into(),
        ReportTab::Weekly => text_input("YYYY-MM-DD", &state.weekly_report_input)
            .on_input(Message::WeeklyReportInputChanged)
            .padding(10)
            .into(),
    };

    let label = match state.report_tab {
        ReportTab::Daily => "Date",
        ReportTab::Weekly => "Week starting",
    };

    Column::new()
        .spacing(12)
        .push(text("Reports").size(24))
        .push(tabs)
        .push(form_field(label, date_input))
        .into()
}

fn view_report_rows(rows: &[ReportRow]) -> Element<'static, Message> {
    let total_minutes: i64 = rows.iter().map(|row| row.total_minutes).sum();
    let total_sessions: usize = rows.iter().map(|row| row.session_count).sum();

    let mut report = Column::new().spacing(16);
    report = report.push(
        Row::new()
            .spacing(16)
            .push(metric_card("Children", rows.len().to_string()))
            .push(metric_card("Sessions", total_sessions.to_string()))
            .push(metric_card("Total Time", reports::format_minutes(total_minutes))),
    );
    
    if rows.is_empty() {
        return report.push(empty_state("No attendance recorded for this period.")).into();
    }
    
    report = report.push(report_table_header());

    for (i, row) in rows.iter().enumerate() {
        let is_even = i % 2 == 0;
        let table_row = Row::new()
            .spacing(16)
            .push(table_cell(row.child_name.clone(), 4))
            .push(table_cell(row.session_count.to_string(), 2))
            .push(table_cell(reports::format_minutes(row.total_minutes), 3))
            .push(table_cell(row.incomplete_sessions.to_string(), 2));

        report = report.push(
            container(table_row)
                .padding([12, 16])
                .width(Fill)
                .style(move |theme: &Theme| table_row_style(theme, is_even, false)),
        );
    }

    report.into()
}

fn metric_card<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    container(
        Column::new()
            .spacing(6)
            .push(
                text(label)
                    .size(13)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(|theme: &Theme| text::Style { color: Some(theme.extended_palette().secondary.base.color) }),
            )
            .push(text(value).size(32)),
    )
    .padding(20)
    .width(Length::FillPortion(1))
    .style(|theme: &Theme| {
        container::background(theme.palette().background)
            .border(iced::border::rounded(12.0))
            .shadow(iced::Shadow {
                color: iced::Color::from_rgba8(0, 0, 0, 0.05),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            })
    })
    .into()
}

fn form_field<'a>(
    label: &'a str,
    input: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        Column::new()
            .spacing(8)
            .push(
                text(label)
                    .size(14)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(|theme: &Theme| text::Style { color: Some(theme.extended_palette().secondary.weak.text) }),
            )
            .push(input),
    )
    .width(Fill)
    .into()
}

fn section_card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(24)
        .width(Fill)
        .style(|theme: &Theme| {
            container::background(theme.palette().background)
                .border(iced::border::rounded(16.0))
                .shadow(iced::Shadow {
                    color: iced::Color::from_rgba8(0, 0, 0, 0.05),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                })
        })
        .into()
}

fn children_table_header() -> Element<'static, Message> {
    table_header([
        ("Child", 3),
        ("Parent", 3),
        ("Birth Date", 2),
        ("Phone", 2),
        ("Status", 2),
        ("Actions", 3),
    ])
}

fn attendance_table_header(action_label: &'static str) -> Element<'static, Message> {
    table_header([
        ("Child", 3),
        ("Parent", 3),
        ("Status", 2),
        ("Last Activity", 3),
        (action_label, 2),
    ])
}

fn recent_sessions_header() -> Element<'static, Message> {
    table_header([
        ("Date", 2),
        ("Check In", 2),
        ("Check Out", 2),
        ("Duration", 3),
    ])
}

fn report_table_header() -> Element<'static, Message> {
    table_header([
        ("Child", 4),
        ("Sessions", 2),
        ("Total Time", 3),
        ("Open", 2),
    ])
}

fn table_header<'a, const N: usize>(columns: [(&'a str, u16); N]) -> Element<'a, Message> {
    let mut row = Row::new().spacing(16);

    for (label, portion) in columns {
        row = row.push(
            container(
                text(label.to_uppercase())
                    .size(12)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .style(|theme: &Theme| text::Style { color: Some(theme.extended_palette().secondary.base.color) }),
            )
            .width(Length::FillPortion(portion)),
        );
    }

    container(row)
        .padding([12, 16])
        .width(Fill)
        .style(|theme: &Theme| {
            container::background(theme.palette().background).border(iced::Border {
                width: 0.0,
                radius: 0.0.into(),
                color: iced::Color::TRANSPARENT,
            })
        })
        .into()
}

fn empty_state<'a>(message: &'a str) -> Element<'a, Message> {
    container(
        text(message)
            .size(16)
            .font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
            .style(|theme: &Theme| text::Style { color: Some(theme.extended_palette().secondary.weak.text) }),
    )
    .width(Fill)
    .padding(60)
    .center_x(Fill)
    .into()
}

fn table_row_style(theme: &Theme, is_even: bool, is_selected: bool) -> container::Style {
    if is_selected {
        container::background(theme.palette().primary)
            .border(iced::border::rounded(8.0))
    } else if is_even {
        container::background(theme.extended_palette().background.weak.color)
            .border(iced::border::rounded(8.0))
    } else {
        container::Style::default()
    }
}

fn table_cell<'a>(value: String, portion: u16) -> Element<'a, Message> {
    container(text(value).size(15))
        .width(Length::FillPortion(portion))
        .into()
}

fn required_value(value: &str, error: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(error.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn parse_date_value(input: &str, error: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(input.trim(), "%Y-%m-%d").map_err(|_| error.to_string())
}
