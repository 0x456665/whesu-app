use crate::{
    attendance,
    models::{AppScreen, AttendanceRecord, ChildRecord, Gender, ParentInfo},
    reports::{self, ReportRow},
    sample_data,
    storage::DataStore,
};
use chrono::{Datelike, Local, NaiveDate};
use iced::{
    application,
    widget::{
        button, column, container, horizontal_rule, pick_list, row, scrollable, text,
        text_input, Row,
    },
    Alignment, Color, Element, Fill, Font, Length, Shadow, Size, Task, Theme, Vector,
};
use std::{cmp::Reverse, fmt};

const PRIMARY: Color = Color::from_rgb(0.173, 0.431, 0.620);
const PRIMARY_DARK: Color = Color::from_rgb(0.122, 0.306, 0.447);
const SUCCESS: Color = Color::from_rgb(0.298, 0.686, 0.314);
const DANGER: Color = Color::from_rgb(0.898, 0.224, 0.208);
const BG: Color = Color::from_rgb(0.961, 0.969, 0.980);
const CARD: Color = Color::WHITE;
const TEXT_MUTED: Color = Color::from_rgb(0.502, 0.502, 0.502);
const ROW_ALT: Color = Color::from_rgb(0.941, 0.949, 0.961);
const SELECTED_ROW: Color = Color::from_rgb(0.863, 0.922, 0.984);
const BORDER_COLOR: Color = Color::from_rgb(0.867, 0.882, 0.902);

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.label) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MonthOption { value: u32, label: &'static str }
impl fmt::Display for MonthOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.label) }
}

const MONTHS: [MonthOption; 12] = [
    MonthOption { value: 1, label: "January" }, MonthOption { value: 2, label: "February" },
    MonthOption { value: 3, label: "March" },   MonthOption { value: 4, label: "April" },
    MonthOption { value: 5, label: "May" },     MonthOption { value: 6, label: "June" },
    MonthOption { value: 7, label: "July" },    MonthOption { value: 8, label: "August" },
    MonthOption { value: 9, label: "September" },MonthOption { value: 10, label: "October" },
    MonthOption { value: 11, label: "November" },MonthOption { value: 12, label: "December" },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainTab { Dashboard, Children, Attendance, Reports, EmergencyContacts, Settings }

impl MainTab {
    const ALL: [Self; 6] = [Self::Dashboard, Self::Children, Self::Attendance,
                             Self::Reports, Self::EmergencyContacts, Self::Settings];
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Children => "Children",
            Self::Attendance => "Attendance",
            Self::Reports => "Reports",
            Self::EmergencyContacts => "Emergency Contacts",
            Self::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttendanceTab { CheckIn, CheckOut }
impl AttendanceTab {
    const ALL: [Self; 2] = [Self::CheckIn, Self::CheckOut];
    fn label(self) -> &'static str {
        match self { Self::CheckIn => "Check In", Self::CheckOut => "Check Out" }
    }
}

/// Sub-page within the Children tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildrenPage { Roster, AddEdit }

#[derive(Debug, Clone, Copy, PartialEq)]
enum ReportType { Daily, Weekly, Monthly }

#[derive(Debug, Clone, Copy)]
enum StatusKind { Success, Error }

#[derive(Debug, Clone)]
struct StatusMessage { kind: StatusKind, text: String }
impl StatusMessage {
    fn success(text: impl Into<String>) -> Self { Self { kind: StatusKind::Success, text: text.into() } }
    fn error(text: impl Into<String>) -> Self { Self { kind: StatusKind::Error, text: text.into() } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildFormMode { Add, Edit(u32) }
impl ChildFormMode {
    fn title(self) -> &'static str { match self { Self::Add => "Add New Child", Self::Edit(_) => "Edit Child Record" } }
    fn submit_label(self) -> &'static str { match self { Self::Add => "Add Child", Self::Edit(_) => "Save Changes" } }
}

#[derive(Debug, Clone, Copy)]
enum ChildFormField {
    FirstName, LastName, DateOfBirth,
    ParentFirstName, ParentLastName, PhoneNumber, Email,
    Address, City, State, ZipCode,
    Allergies, EmergencyContactName, EmergencyContactPhone,
}

#[derive(Debug, Clone)]
struct ChildForm {
    first_name: String, last_name: String, date_of_birth: String, gender: Gender,
    parent_first_name: String, parent_last_name: String, phone_number: String, email: String,
    address: String, city: String, state: String, zip_code: String,
    allergies: String, emergency_contact_name: String, emergency_contact_phone: String,
    phone_error: Option<String>, ec_phone_error: Option<String>,
}

impl Default for ChildForm {
    fn default() -> Self {
        Self {
            first_name: String::new(), last_name: String::new(), date_of_birth: String::new(),
            gender: Gender::PreferNotToSay,
            parent_first_name: String::new(), parent_last_name: String::new(),
            phone_number: String::new(), email: String::new(),
            address: String::new(), city: String::new(), state: "TX".to_string(), zip_code: String::new(),
            allergies: String::new(), emergency_contact_name: String::new(),
            emergency_contact_phone: String::new(),
            phone_error: None, ec_phone_error: None,
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
            ChildFormField::PhoneNumber => { self.phone_error = validate_phone_field(&value); self.phone_number = value; }
            ChildFormField::Email => self.email = value,
            ChildFormField::Address => self.address = value,
            ChildFormField::City => self.city = value,
            ChildFormField::State => self.state = value,
            ChildFormField::ZipCode => self.zip_code = value,
            ChildFormField::Allergies => self.allergies = value,
            ChildFormField::EmergencyContactName => self.emergency_contact_name = value,
            ChildFormField::EmergencyContactPhone => { self.ec_phone_error = validate_phone_field(&value); self.emergency_contact_phone = value; }
        }
    }

    fn build_child(&self, next_id: u32) -> Result<ChildRecord, String> {
        let first_name = required_value(&self.first_name, "Enter the child's first name.")?;
        let last_name = required_value(&self.last_name, "Enter the child's last name.")?;
        let parent_first_name = required_value(&self.parent_first_name, "Enter the parent's first name.")?;
        let parent_last_name = required_value(&self.parent_last_name, "Enter the parent's last name.")?;
        let phone_number = required_value(&self.phone_number, "Enter a phone number.")?;
        if validate_phone_field(&phone_number).is_some() {
            return Err("Parent phone must be in format +1(NNN)-NNN-NNNN.".to_string());
        }
        if !self.emergency_contact_phone.is_empty() && validate_phone_field(&self.emergency_contact_phone).is_some() {
            return Err("Emergency contact phone must be in format +1(NNN)-NNN-NNNN.".to_string());
        }
        let date_of_birth = parse_date_value(&self.date_of_birth, "Enter the date of birth as YYYY-MM-DD.")?;
        Ok(ChildRecord {
            id: next_id, first_name, last_name, date_of_birth, gender: self.gender,
            parent: ParentInfo {
                first_name: parent_first_name, last_name: parent_last_name,
                address: self.address.trim().to_string(), city: self.city.trim().to_string(),
                state: self.state.trim().to_string(), zip_code: self.zip_code.trim().to_string(),
                phone_number, email: self.email.trim().to_string(),
            },
            allergies: self.allergies.trim().to_string(),
            emergency_contact_name: self.emergency_contact_name.trim().to_string(),
            emergency_contact_phone: self.emergency_contact_phone.trim().to_string(),
        })
    }

    fn from_child(child: &ChildRecord) -> Self {
        Self {
            first_name: child.first_name.clone(), last_name: child.last_name.clone(),
            date_of_birth: child.date_of_birth.format("%Y-%m-%d").to_string(),
            gender: child.gender,
            parent_first_name: child.parent.first_name.clone(), parent_last_name: child.parent.last_name.clone(),
            phone_number: child.parent.phone_number.clone(), email: child.parent.email.clone(),
            address: child.parent.address.clone(), city: child.parent.city.clone(),
            state: child.parent.state.clone(), zip_code: child.parent.zip_code.clone(),
            allergies: child.allergies.clone(),
            emergency_contact_name: child.emergency_contact_name.clone(),
            emergency_contact_phone: child.emergency_contact_phone.clone(),
            phone_error: None, ec_phone_error: None,
        }
    }
}

fn validate_phone_field(phone: &str) -> Option<String> {
    if phone.is_empty() { return None; }
    if is_valid_phone(phone) { None } else { Some("Format must be +1(NNN)-NNN-NNNN".to_string()) }
}

fn is_valid_phone(phone: &str) -> bool {
    let c: Vec<char> = phone.chars().collect();
    if c.len() != 16 { return false; }
    c[0]=='+' && c[1]=='1' && c[2]=='(' &&
    c[3].is_ascii_digit() && c[4].is_ascii_digit() && c[5].is_ascii_digit() &&
    c[6]==')' && c[7]=='-' &&
    c[8].is_ascii_digit() && c[9].is_ascii_digit() && c[10].is_ascii_digit() &&
    c[11]=='-' &&
    c[12].is_ascii_digit() && c[13].is_ascii_digit() && c[14].is_ascii_digit() && c[15].is_ascii_digit()
}

fn required_value(value: &str, error: &str) -> Result<String, String> {
    let t = value.trim();
    if t.is_empty() { Err(error.to_string()) } else { Ok(t.to_string()) }
}

fn parse_date_value(value: &str, error: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|_| error.to_string())
}

pub(crate) struct DaycareApp {
    screen: AppScreen,
    login_password: String,
    login_error: Option<String>,
    store: DataStore,
    children: Vec<ChildRecord>,
    attendance_records: Vec<AttendanceRecord>,
    active_tab: MainTab,
    attendance_tab: AttendanceTab,
    children_page: ChildrenPage,
    selected_child_id: Option<u32>,
    child_form_mode: ChildFormMode,
    child_form: ChildForm,
    pending_delete_id: Option<u32>,
    report_type: ReportType,
    report_daily_input: String,
    report_weekly_input: String,
    report_month: MonthOption,
    report_year: String,
    report_child_filter: Option<u32>,
    report_rows: Vec<ReportRow>,
    report_generated: bool,
    report_export_path: String,
    report_export_status: Option<String>,
    ec_search: String,
    settings_current_pw: String,
    settings_new_pw: String,
    settings_confirm_pw: String,
    settings_status: Option<StatusMessage>,
    status: Option<StatusMessage>,
}

impl Default for DaycareApp {
    fn default() -> Self {
        let store = DataStore::new_seeded().expect("failed to initialize SQLite datastore");
        let children = store.load_children().expect("failed to load children");
        let attendance_records = store.load_attendance().unwrap_or_else(|_| sample_data::sample_attendance());
        let now = Local::now();
        let current_month = MONTHS[(now.month() as usize).saturating_sub(1)].clone();
        Self {
            screen: AppScreen::Login,
            login_password: String::new(), login_error: None,
            store, children, attendance_records,
            active_tab: MainTab::Dashboard,
            attendance_tab: AttendanceTab::CheckIn,
            children_page: ChildrenPage::Roster,
            selected_child_id: None,
            child_form_mode: ChildFormMode::Add,
            child_form: ChildForm::default(),
            pending_delete_id: None,
            report_type: ReportType::Daily,
            report_daily_input: sample_data::default_daily_report_date().format("%Y-%m-%d").to_string(),
            report_weekly_input: sample_data::default_week_start().format("%Y-%m-%d").to_string(),
            report_month: current_month,
            report_year: now.year().to_string(),
            report_child_filter: None,
            report_rows: Vec::new(),
            report_generated: false,
            report_export_path: String::new(),
            report_export_status: None,
            ec_search: String::new(),
            settings_current_pw: String::new(), settings_new_pw: String::new(),
            settings_confirm_pw: String::new(), settings_status: None,
            status: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    LoginPasswordChanged(String), LoginSubmit, Logout,
    TabSelected(MainTab), AttendanceTabSelected(AttendanceTab),
    ShowChildRoster, ShowAddChildForm,
    ChildSelected(u32), OpenAttendanceForChild(u32),
    EditChild(u32), RequestDeleteChild(u32), ConfirmDelete, CancelDelete,
    ResetChildForm, ChildFormChanged(ChildFormField, String), ChildGenderSelected(Gender), SaveChild,
    CheckInSelectedChild, CheckOutSelectedChild, CheckInChild(u32), CheckOutChild(u32),
    ReportTypeSelected(ReportType), ReportDailyInputChanged(String), ReportWeeklyInputChanged(String),
    ReportMonthSelected(MonthOption), ReportYearChanged(String),
    ReportChildFilterSelected(ChildOption), ReportChildFilterCleared,
    GenerateReport, ReportExportPathChanged(String), ExportPickFolder, ExportFolderPicked(String), ExportCsv,
    EcSearchChanged(String),
    SettingsCurrentPwChanged(String), SettingsNewPwChanged(String), SettingsConfirmPwChanged(String),
    ChangePassword,
    Noop,
}

pub(crate) fn run() -> iced::Result {
    application("Happy Haven Daycare", update, view)
        .theme(|_| Theme::Light)
        .window_size(Size::new(1280.0, 900.0))
        .run()
}

fn update(state: &mut DaycareApp, message: Message) -> Task<Message> {
    match message {
        Message::LoginPasswordChanged(v) => state.login_password = v,
        Message::LoginSubmit => {
            if state.store.verify_password(&state.login_password) {
                state.screen = AppScreen::Main;
                state.login_password.clear();
                state.login_error = None;
            } else {
                state.login_error = Some("Incorrect password.".to_string());
                state.login_password.clear();
            }
        }
        Message::Logout => {
            state.screen = AppScreen::Login;
            state.login_password.clear();
            state.login_error = None;
            state.status = None;
        }
        Message::TabSelected(tab) => {
            state.active_tab = tab;
            state.status = None;
            if tab == MainTab::Children { state.children_page = ChildrenPage::Roster; }
        }
        Message::AttendanceTabSelected(tab) => state.attendance_tab = tab,
        Message::ShowChildRoster => {
            state.reset_child_form();
            state.children_page = ChildrenPage::Roster;
        }
        Message::ShowAddChildForm => {
            state.reset_child_form();
            state.active_tab = MainTab::Children;
            state.children_page = ChildrenPage::AddEdit;
        }
        Message::ChildSelected(id) => state.selected_child_id = Some(id),
        Message::OpenAttendanceForChild(id) => {
            state.selected_child_id = Some(id);
            state.active_tab = MainTab::Attendance;
            state.attendance_tab = if attendance::is_checked_in(&state.attendance_records, id) {
                AttendanceTab::CheckOut } else { AttendanceTab::CheckIn };
        }
        Message::EditChild(id) => state.start_editing_child(id),
        Message::RequestDeleteChild(id) => state.pending_delete_id = Some(id),
        Message::ConfirmDelete => { if let Some(id) = state.pending_delete_id.take() { state.delete_child(id); } }
        Message::CancelDelete => state.pending_delete_id = None,
        Message::ResetChildForm => {
            state.reset_child_form();
            state.children_page = ChildrenPage::Roster;
        }
        Message::ChildFormChanged(field, value) => state.child_form.set(field, value),
        Message::ChildGenderSelected(g) => state.child_form.gender = g,
        Message::SaveChild => state.save_child(),
        Message::CheckInSelectedChild => match state.selected_child_id {
            Some(id) => state.check_in_child(id),
            None => state.set_status(StatusMessage::error("Select a child to check in.")),
        },
        Message::CheckOutSelectedChild => match state.selected_child_id {
            Some(id) => state.check_out_child(id),
            None => state.set_status(StatusMessage::error("Select a child to check out.")),
        },
        Message::CheckInChild(id) => state.check_in_child(id),
        Message::CheckOutChild(id) => state.check_out_child(id),
        Message::ReportTypeSelected(rt) => {
            state.report_type = rt; state.report_generated = false; state.report_rows.clear();
        }
        Message::ReportDailyInputChanged(v) => state.report_daily_input = v,
        Message::ReportWeeklyInputChanged(v) => state.report_weekly_input = v,
        Message::ReportMonthSelected(m) => state.report_month = m,
        Message::ReportYearChanged(v) => state.report_year = v,
        Message::ReportChildFilterSelected(opt) => { state.report_child_filter = Some(opt.id); state.report_generated = false; }
        Message::ReportChildFilterCleared => { state.report_child_filter = None; state.report_generated = false; }
        Message::GenerateReport => state.generate_report(),
        Message::ReportExportPathChanged(v) => state.report_export_path = v,
        Message::ExportPickFolder => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Choose export folder")
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_string_lossy().to_string())
                },
                |opt| match opt {
                    Some(dir) => {
                        let ts = chrono::Local::now().format("%Y-%m-%d").to_string();
                        Message::ExportFolderPicked(format!("{dir}/whesu_report_{ts}.csv"))
                    }
                    None => Message::Noop,
                },
            );
        }
        Message::ExportFolderPicked(path) => {
            state.report_export_path = path;
        }
        Message::ExportCsv => state.export_csv(),
        Message::EcSearchChanged(v) => state.ec_search = v,
        Message::SettingsCurrentPwChanged(v) => state.settings_current_pw = v,
        Message::SettingsNewPwChanged(v) => state.settings_new_pw = v,
        Message::SettingsConfirmPwChanged(v) => state.settings_confirm_pw = v,
        Message::ChangePassword => state.change_password(),
        Message::Noop => {}
    }
    Task::none()
}

fn view(state: &DaycareApp) -> Element<'_, Message> {
    match state.screen {
        AppScreen::Login => view_login(state),
        AppScreen::Main => view_main(state),
    }
}

fn view_login(state: &DaycareApp) -> Element<'_, Message> {
    let logo = column![
        text("Happy Haven Daycare").size(36)
            .font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
            .style(|_: &Theme| text::Style { color: Some(PRIMARY) }),
        text("Childcare Management System").size(16)
            .style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
    ].spacing(6).align_x(Alignment::Center);

    let mut form = column![
        text("Sign In").size(22).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        text("Enter your password to continue.").size(14)
            .style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
        column![
            text("Password").size(13).font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
                .style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
            text_input("Password", &state.login_password)
                .secure(true)
                .on_input(Message::LoginPasswordChanged)
                .on_submit(Message::LoginSubmit)
                .padding([12, 14])
                .width(Fill),
        ].spacing(6),
    ].spacing(16);

    if let Some(err) = &state.login_error {
        form = form.push(
            container(text(err.as_str()).size(14).style(|_: &Theme| text::Style { color: Some(Color::WHITE) }))
                .padding([10, 14]).width(Fill)
                .style(|_: &Theme| container::Style {
                    background: Some(iced::Background::Color(DANGER)),
                    border: iced::border::rounded(8.0), ..Default::default()
                }),
        );
    }

    form = form.push(
        button(text("Sign In").size(16).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }))
            .padding([14, 0]).width(Fill).style(primary_button_style).on_press(Message::LoginSubmit),
    );

    let inner_card = container(column![logo, form].spacing(32).max_width(420))
        .padding(44)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(CARD)),
            border: iced::border::rounded(20.0),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 32.0,
            },
            ..Default::default()
        });

    container(inner_card)
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(BG)),
            ..Default::default()
        })
        .into()
}

fn view_main(state: &DaycareApp) -> Element<'_, Message> {
    row![
        view_sidebar(state),
        column![
            view_topbar(state),
            scrollable(container(view_tab_content(state)).padding([24, 28]).width(Fill)).height(Fill),
        ].height(Fill),
    ]
    .height(Fill)
    .into()
}

fn view_sidebar(state: &DaycareApp) -> Element<'_, Message> {
    let logo_badge = container(
        text("HH").size(22).font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
            .style(|_: &Theme| text::Style { color: Some(PRIMARY) })
    )
    .width(48).height(48)
    .center_x(48).center_y(48)
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(Color::WHITE)),
        border: iced::Border { radius: 14.0.into(), width: 0.0, color: Color::TRANSPARENT },
        ..Default::default()
    });

    let title = column![
        logo_badge,
        text("Happy Haven").size(18).font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
            .style(|_: &Theme| text::Style { color: Some(Color::WHITE) }),
        text("Daycare").size(13)
            .style(|_: &Theme| text::Style { color: Color::from_rgba(1.0, 1.0, 1.0, 0.7).into() }),
    ].spacing(6).align_x(Alignment::Center)
     .padding(iced::Padding { top: 20.0, right: 20.0, bottom: 16.0, left: 20.0 });

    let mut nav = column![title, horizontal_rule(1)].spacing(0);

    for tab in MainTab::ALL {
        let is_active = state.active_tab == tab;
        nav = nav.push(
            button(text(tab.label()).size(15).style(move |_: &Theme| {
                if is_active { text::Style { color: Some(Color::WHITE) } }
                else { text::Style { color: Color::from_rgba(1.0, 1.0, 1.0, 0.75).into() } }
            }))
            .padding([13, 20]).width(Fill)
            .style(move |_: &Theme, _: button::Status| {
                if is_active {
                    button::Style {
                        background: Some(iced::Background::Color(PRIMARY_DARK)),
                        text_color: Color::WHITE,
                        border: iced::border::rounded(0.0),
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        background: None,
                        text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.75),
                        border: iced::border::rounded(0.0),
                        ..Default::default()
                    }
                }
            })
            .on_press(Message::TabSelected(tab)),
        );
    }

    nav = nav.push(
        container(
            button(text("Logout").size(14)
                .style(|_: &Theme| text::Style { color: Color::from_rgba(1.0, 0.6, 0.6, 1.0).into() }))
                .padding([10, 20]).width(Fill)
                .style(|_: &Theme, _: button::Status| button::Style { background: None, ..Default::default() })
                .on_press(Message::Logout),
        ).padding(iced::Padding { top: 8.0, right: 0.0, bottom: 12.0, left: 0.0 }),
    );

    container(nav).width(210).height(Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(PRIMARY)),
            ..Default::default()
        })
        .into()
}

fn view_topbar(state: &DaycareApp) -> Element<'_, Message> {
    let date_str = Local::now().format("%A, %B %-d, %Y").to_string();

    let page_title: &str = match (state.active_tab, state.children_page) {
        (MainTab::Children, ChildrenPage::AddEdit) => match state.child_form_mode {
            ChildFormMode::Add => "Children  /  Add New Child",
            ChildFormMode::Edit(_) => "Children  /  Edit Child",
        },
        _ => state.active_tab.label(),
    };

    let bar = row![
        text(page_title).size(20).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        container(row![]).width(Fill),
        text(date_str).size(13).style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
    ].align_y(Alignment::Center).spacing(16).padding([14, 24]);

    let bar = if let Some(status) = &state.status {
        let (bg, tc) = match status.kind {
            StatusKind::Success => (SUCCESS, Color::WHITE),
            StatusKind::Error => (DANGER, Color::WHITE),
        };
        let msg_text = status.text.clone();
        bar.push(
            container(text(format!("  {}  ", &msg_text)).size(13)
                .style(move |_: &Theme| text::Style { color: Some(tc) }))
                .padding([6, 14])
                .style(move |_: &Theme| container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::border::rounded(20.0),
                    ..Default::default()
                }),
        )
    } else {
        bar
    };

    container(bar).width(Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(CARD)),
            border: iced::Border { width: 1.0, color: BORDER_COLOR, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
}

fn view_tab_content(state: &DaycareApp) -> Element<'_, Message> {
    match state.active_tab {
        MainTab::Dashboard => view_dashboard(state),
        MainTab::Children => view_children_tab(state),
        MainTab::Attendance => view_attendance_tab(state),
        MainTab::Reports => view_reports_tab(state),
        MainTab::EmergencyContacts => view_emergency_contacts_tab(state),
        MainTab::Settings => view_settings_tab(state),
    }
}

// ─────────────────── Dashboard ──────────────────────────────────────────────

fn view_dashboard(state: &DaycareApp) -> Element<'_, Message> {
    let checked_in = state.currently_checked_in_count();
    let checked_out = state.children.len().saturating_sub(checked_in);

    let stat_row = row![
        stat_card("Total Enrolled", state.children.len().to_string(), PRIMARY),
        stat_card("Checked In Now", checked_in.to_string(), SUCCESS),
        stat_card("Checked Out", checked_out.to_string(), DANGER),
    ].spacing(16);

    let quick_actions = card("Quick Actions", column![
        text("Jump to a common task:").size(14)
            .style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
        row![
            action_button("Check In / Out", Message::TabSelected(MainTab::Attendance)),
            action_button("Add a Child", Message::ShowAddChildForm),
            action_button("Run a Report", Message::TabSelected(MainTab::Reports)),
            action_button("Emergency Contacts", Message::TabSelected(MainTab::EmergencyContacts)),
        ].spacing(10),
    ].spacing(12));

    let checked_in_children: Vec<_> = state.children.iter()
        .filter(|c| attendance::is_checked_in(&state.attendance_records, c.id)).collect();

    let mut list_col = column![section_heading("Currently Checked In")].spacing(10);
    if checked_in_children.is_empty() {
        list_col = list_col.push(empty_state("No children are currently checked in."));
    } else {
        list_col = list_col.push(table_header_row(&[("Child", 3), ("Parent", 3), ("Checked In At", 3)]));
        for (i, child) in checked_in_children.iter().enumerate() {
            let ci_time = state.attendance_records.iter()
                .filter(|r| r.child_id == child.id && r.check_out.is_none())
                .map(|r| r.check_in.format("%I:%M %p").to_string())
                .next().unwrap_or_default();
            list_col = list_col.push(table_row(
                &[(child.full_name(), 3), (child.parent.full_name(), 3), (ci_time, 3)],
                i % 2 == 0, false,
            ));
        }
    }

    column![stat_row, quick_actions, card_element(list_col.into())].spacing(20).into()
}

// ─────────────────── Children Tab ──────────────────────────────────────────

fn view_children_tab(state: &DaycareApp) -> Element<'_, Message> {
    match state.children_page {
        ChildrenPage::Roster => view_children_roster(state),
        ChildrenPage::AddEdit => view_child_form_page(state),
    }
}

fn view_children_roster(state: &DaycareApp) -> Element<'_, Message> {
    let mut layout = column![].spacing(20);

    // Delete confirmation banner
    if let Some(child_id) = state.pending_delete_id {
        let cname = state.child_label(child_id);
        layout = layout.push(
            container(column![
                text("Confirm Deletion").size(18)
                    .font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
                text(format!("Delete {}? All attendance records will also be removed.", cname)).size(14),
                row![
                    button(text("Delete Permanently").size(14)).padding([10, 18])
                        .style(danger_button_style).on_press(Message::ConfirmDelete),
                    button(text("Cancel").size(14)).padding([10, 18])
                        .style(secondary_button_style).on_press(Message::CancelDelete),
                ].spacing(12),
            ].spacing(12))
            .padding(20)
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(1.0, 0.95, 0.95))),
                border: iced::Border { width: 2.0, color: DANGER, radius: 12.0.into() },
                ..Default::default()
            }),
        );
    }

    // Header: title + Add Child button
    let header = row![
        text("Children Roster").size(20)
            .font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        container(row![]).width(Fill),
        button(text("+ Add New Child").size(14)
            .font(Font { weight: iced::font::Weight::Bold, ..Default::default() }))
            .padding([10, 20]).style(primary_button_style).on_press(Message::ShowAddChildForm),
    ].align_y(Alignment::Center).spacing(12);

    // Table
    let mut table = column![header].spacing(10);

    if state.children.is_empty() {
        table = table.push(empty_state(
            "No children enrolled. Click \"+ Add New Child\" to get started."
        ));
    } else {
        table = table.push(table_header_row(&[
            ("Child", 3), ("Parent", 3), ("Date of Birth", 2), ("Phone", 3), ("Status", 2), ("Actions", 4),
        ]));
        for (i, child) in state.children.iter().enumerate() {
            let is_selected = state.selected_child_id == Some(child.id);
            let is_even = i % 2 == 0;
            let sl = state.status_label(child.id);
            let cid = child.id;
            let actions_row = row![
                button(text("Attend").size(13)).padding([6, 10])
                    .style(secondary_button_style).on_press(Message::OpenAttendanceForChild(cid)),
                button(text("Edit").size(13)).padding([6, 10])
                    .style(secondary_button_style).on_press(Message::EditChild(cid)),
                button(text("Delete").size(13)).padding([6, 10])
                    .style(danger_button_style).on_press(Message::RequestDeleteChild(cid)),
            ].spacing(6);
            let row_elem = row![
                table_cell_elem(child.full_name(), 3),
                table_cell_elem(child.parent.full_name(), 3),
                table_cell_elem(child.date_of_birth.format("%Y-%m-%d").to_string(), 2),
                table_cell_elem(child.parent.phone_number.clone(), 3),
                container(text(sl).size(13).style(move |_: &Theme| {
                    if sl == "Checked In" { text::Style { color: Some(SUCCESS) } }
                    else { text::Style { color: Some(TEXT_MUTED) } }
                })).width(Length::FillPortion(2)),
                container(actions_row).width(Length::FillPortion(4)),
            ].spacing(12).align_y(Alignment::Center);
            table = table.push(
                container(row_elem).padding([10, 14]).width(Fill)
                    .style(move |_: &Theme| table_row_bg(is_even, is_selected))
            );
        }
    }

    layout.push(card_element(table.into())).into()
}

fn view_child_form_page(state: &DaycareApp) -> Element<'_, Message> {
    let back_btn = button(
        text("<- Back to Roster").size(14)
            .style(|_: &Theme| text::Style { color: Some(PRIMARY) })
    )
    .padding([0, 0])
    .style(|_: &Theme, _: button::Status| button::Style { background: None, ..Default::default() })
    .on_press(Message::ShowChildRoster);

    let heading = row![
        back_btn,
    ].align_y(Alignment::Center).spacing(0);

    column![
        heading,
        card(state.child_form_mode.title(), view_child_form(state)),
    ].spacing(16).into()
}

fn view_child_form(state: &DaycareApp) -> Element<'_, Message> {
    let gender_picker = pick_list(GENDERS, Some(state.child_form.gender), Message::ChildGenderSelected)
        .placeholder("Select gender").width(Fill);

    let actions = row![
        button(text(state.child_form_mode.submit_label())).padding([10, 20])
            .style(primary_button_style).on_press(Message::SaveChild),
        button(text("Cancel")).padding([10, 20])
            .style(secondary_button_style).on_press(Message::ShowChildRoster),
    ].spacing(10);

    let phone_col: Element<'_, Message> = {
        let mut c = column![
            text_input("+1(NNN)-NNN-NNNN", &state.child_form.phone_number)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::PhoneNumber, v))
                .padding(10).width(Fill),
        ].spacing(4);
        if let Some(e) = &state.child_form.phone_error {
            c = c.push(text(e.as_str()).size(12).style(|_: &Theme| text::Style { color: Some(DANGER) }));
        }
        c.into()
    };

    let ec_phone_col: Element<'_, Message> = {
        let mut c = column![
            text_input("+1(NNN)-NNN-NNNN", &state.child_form.emergency_contact_phone)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::EmergencyContactPhone, v))
                .padding(10).width(Fill),
        ].spacing(4);
        if let Some(e) = &state.child_form.ec_phone_error {
            c = c.push(text(e.as_str()).size(12).style(|_: &Theme| text::Style { color: Some(DANGER) }));
        }
        c.into()
    };

    column![
        subsection_heading("Child Information"),
        row![
            form_field("First Name", text_input("First name", &state.child_form.first_name)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::FirstName, v))
                .padding(10).width(Fill)),
            form_field("Last Name", text_input("Last name", &state.child_form.last_name)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::LastName, v))
                .padding(10).width(Fill)),
        ].spacing(12),
        row![
            form_field("Date of Birth", text_input("YYYY-MM-DD", &state.child_form.date_of_birth)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::DateOfBirth, v))
                .padding(10).width(Fill)),
            form_field("Gender", gender_picker),
        ].spacing(12),

        subsection_heading("Parent / Guardian"),
        row![
            form_field("First Name", text_input("First name", &state.child_form.parent_first_name)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::ParentFirstName, v))
                .padding(10).width(Fill)),
            form_field("Last Name", text_input("Last name", &state.child_form.parent_last_name)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::ParentLastName, v))
                .padding(10).width(Fill)),
        ].spacing(12),
        row![
            form_field("Phone (+1(NNN)-NNN-NNNN)", phone_col),
            form_field("Email", text_input("Email address", &state.child_form.email)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::Email, v))
                .padding(10).width(Fill)),
        ].spacing(12),
        form_field("Street Address", text_input("Street address", &state.child_form.address)
            .on_input(|v| Message::ChildFormChanged(ChildFormField::Address, v))
            .padding(10).width(Fill)),
        row![
            form_field("City", text_input("City", &state.child_form.city)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::City, v))
                .padding(10).width(Fill)),
            form_field("State", text_input("State", &state.child_form.state)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::State, v))
                .padding(10).width(Fill)),
            form_field("Zip Code", text_input("Zip code", &state.child_form.zip_code)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::ZipCode, v))
                .padding(10).width(Fill)),
        ].spacing(12),

        subsection_heading("Emergency Contact"),
        row![
            form_field("Contact Name", text_input("Full name", &state.child_form.emergency_contact_name)
                .on_input(|v| Message::ChildFormChanged(ChildFormField::EmergencyContactName, v))
                .padding(10).width(Fill)),
            form_field("Contact Phone (+1(NNN)-NNN-NNNN)", ec_phone_col),
        ].spacing(12),

        subsection_heading("Medical Notes"),
        form_field("Allergies / Dietary Notes", text_input(
            "e.g. Peanuts, Dairy -- or None",
            &state.child_form.allergies
        ).on_input(|v| Message::ChildFormChanged(ChildFormField::Allergies, v)).padding(10).width(Fill)),

        actions,
    ].spacing(12).into()
}

// ─────────────────── Attendance ─────────────────────────────────────────────

fn view_attendance_tab(state: &DaycareApp) -> Element<'_, Message> {
    let subtabs = row(AttendanceTab::ALL.map(|tab| {
        let is_active = tab == state.attendance_tab;
        button(text(tab.label()).size(14)).padding([9, 18])
            .style(if is_active { primary_button_style } else { secondary_button_style })
            .on_press(Message::AttendanceTabSelected(tab)).into()
    })).spacing(10);

    let picker = pick_list(state.child_options(), state.selected_child_option(), |opt| Message::ChildSelected(opt.id))
        .placeholder("Select a child...").width(Fill);

    let action_btn = match state.attendance_tab {
        AttendanceTab::CheckIn => {
            let enabled = state.selected_child_id
                .is_some_and(|id| !attendance::is_checked_in(&state.attendance_records, id));
            button(text("Check In").size(15).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }))
                .padding([12, 24]).style(success_button_style)
                .on_press_maybe(enabled.then_some(Message::CheckInSelectedChild))
        }
        AttendanceTab::CheckOut => {
            let enabled = state.selected_child_id
                .is_some_and(|id| attendance::is_checked_in(&state.attendance_records, id));
            button(text("Check Out").size(15).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }))
                .padding([12, 24]).style(danger_button_style)
                .on_press_maybe(enabled.then_some(Message::CheckOutSelectedChild))
        }
    };

    let actions_card = card(state.attendance_tab.label(), column![
        subtabs,
        row![
            form_field("Selected Child", picker),
            container(action_btn)
                .padding(iced::Padding { top: 20.0, right: 0.0, bottom: 0.0, left: 0.0 }),
        ].spacing(12).align_y(Alignment::Start),
    ].spacing(16));

    let summary = view_attendance_child_summary(state);
    let table_card = match state.attendance_tab {
        AttendanceTab::CheckIn => view_check_in_table(state),
        AttendanceTab::CheckOut => view_check_out_table(state),
    };
    column![actions_card, summary, table_card].spacing(20).into()
}

fn view_attendance_child_summary(state: &DaycareApp) -> Element<'_, Message> {
    let mut content = column![section_heading("Selected Child")].spacing(10);
    if let Some(child) = state.selected_child() {
        let sl = state.status_label(child.id);
        let accent = if attendance::is_checked_in(&state.attendance_records, child.id) { SUCCESS } else { TEXT_MUTED };
        content = content.push(row![
            stat_card("Name", child.full_name(), PRIMARY),
            stat_card("Status", sl.to_string(), accent),
            stat_card("Parent", child.parent.full_name(), PRIMARY),
            stat_card("Phone", child.parent.phone_number.clone(), TEXT_MUTED),
        ].spacing(12));
        let recent = state.recent_records_for_child(child.id);
        if recent.is_empty() {
            content = content.push(empty_state("No attendance history yet."));
        } else {
            content = content.push(table_header_row(&[("Date", 2), ("Check In", 2), ("Check Out", 2), ("Duration", 3)]));
            for (i, record) in recent.iter().enumerate() {
                let co = record.check_out.map(|t| t.format("%I:%M %p").to_string())
                    .unwrap_or_else(|| "Open".to_string());
                let dur = attendance::duration_minutes(record)
                    .map(reports::format_minutes).unwrap_or_else(|| "Open".to_string());
                content = content.push(table_row(&[
                    (record.check_in.date().format("%Y-%m-%d").to_string(), 2),
                    (record.check_in.format("%I:%M %p").to_string(), 2),
                    (co, 2), (dur, 3),
                ], i % 2 == 0, false));
            }
        }
    } else {
        content = content.push(empty_state("Select a child above to view recent sessions."));
    }
    card_element(content.into())
}

fn view_check_in_table(state: &DaycareApp) -> Element<'_, Message> {
    let available: Vec<_> = state.children.iter()
        .filter(|c| !attendance::is_checked_in(&state.attendance_records, c.id)).collect();
    let mut content = column![section_heading("Ready to Check In")].spacing(10);
    if available.is_empty() {
        return card_element(content.push(empty_state("All children are currently checked in.")).into());
    }
    content = content.push(table_header_row(&[("Child", 3), ("Parent", 3), ("Last Activity", 4), ("Action", 2)]));
    for (i, child) in available.iter().enumerate() {
        let cid = child.id;
        let row_elem = row![
            table_cell_elem(child.full_name(), 3),
            table_cell_elem(child.parent.full_name(), 3),
            table_cell_elem(state.last_activity_label(child.id), 4),
            container(button(text("Check In").size(13)).padding([6, 14])
                .style(success_button_style).on_press(Message::CheckInChild(cid)))
                .width(Length::FillPortion(2)),
        ].spacing(12).align_y(Alignment::Center);
        content = content.push(
            container(row_elem).padding([10, 14]).width(Fill)
                .style(move |_: &Theme| table_row_bg(i % 2 == 0, state.selected_child_id == Some(cid)))
        );
    }
    card_element(content.into())
}

fn view_check_out_table(state: &DaycareApp) -> Element<'_, Message> {
    let checked_in: Vec<_> = state.children.iter()
        .filter(|c| attendance::is_checked_in(&state.attendance_records, c.id)).collect();
    let mut content = column![section_heading("Currently Checked In")].spacing(10);
    if checked_in.is_empty() {
        return card_element(content.push(empty_state("No children are currently checked in.")).into());
    }
    content = content.push(table_header_row(&[("Child", 3), ("Parent", 3), ("Checked In At", 4), ("Action", 2)]));
    for (i, child) in checked_in.iter().enumerate() {
        let cid = child.id;
        let ci_time = state.attendance_records.iter()
            .filter(|r| r.child_id == child.id && r.check_out.is_none())
            .map(|r| r.check_in.format("%I:%M %p").to_string())
            .next().unwrap_or_default();
        let row_elem = row![
            table_cell_elem(child.full_name(), 3),
            table_cell_elem(child.parent.full_name(), 3),
            table_cell_elem(ci_time, 4),
            container(button(text("Check Out").size(13)).padding([6, 14])
                .style(danger_button_style).on_press(Message::CheckOutChild(cid)))
                .width(Length::FillPortion(2)),
        ].spacing(12).align_y(Alignment::Center);
        content = content.push(
            container(row_elem).padding([10, 14]).width(Fill)
                .style(move |_: &Theme| table_row_bg(i % 2 == 0, state.selected_child_id == Some(cid)))
        );
    }
    card_element(content.into())
}

// ─────────────────── Reports ────────────────────────────────────────────────

fn view_reports_tab(state: &DaycareApp) -> Element<'_, Message> {
    let type_section = column![
        text("1. Report Type").size(16).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        row![
            rtype_btn("Daily", ReportType::Daily, state.report_type),
            rtype_btn("Weekly", ReportType::Weekly, state.report_type),
            rtype_btn("Monthly", ReportType::Monthly, state.report_type),
        ].spacing(10),
    ].spacing(10);

    let period_section: Element<'_, Message> = match state.report_type {
        ReportType::Daily => column![
            text("2. Date").size(16).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            form_field("Date (YYYY-MM-DD)", text_input("YYYY-MM-DD", &state.report_daily_input)
                .on_input(Message::ReportDailyInputChanged).padding(10).width(Fill)),
        ].spacing(10).into(),
        ReportType::Weekly => column![
            text("2. Week Start").size(16).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            form_field("Week starting (YYYY-MM-DD)", text_input("YYYY-MM-DD", &state.report_weekly_input)
                .on_input(Message::ReportWeeklyInputChanged).padding(10).width(Fill)),
        ].spacing(10).into(),
        ReportType::Monthly => column![
            text("2. Month & Year").size(16).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
            row![
                form_field("Month", pick_list(MONTHS, Some(state.report_month.clone()), Message::ReportMonthSelected).width(Fill)),
                form_field("Year", text_input("YYYY", &state.report_year)
                    .on_input(Message::ReportYearChanged).padding(10).width(Fill)),
            ].spacing(12),
        ].spacing(10).into(),
    };

    let mut all_opts = vec![ChildOption { id: 0, label: "All Children".to_string() }];
    all_opts.extend(state.child_options());
    let sel_opt = state.report_child_filter
        .map(|id| ChildOption { id, label: state.child_label(id) })
        .unwrap_or(ChildOption { id: 0, label: "All Children".to_string() });

    let filter_section = column![
        text("3. Child Filter (optional)").size(16).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        form_field("Child", pick_list(all_opts, Some(sel_opt), |opt| {
            if opt.id == 0 { Message::ReportChildFilterCleared } else { Message::ReportChildFilterSelected(opt) }
        }).width(Fill)),
    ].spacing(10);

    let gen_section = column![
        text("4. Generate").size(16).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        button(text("Generate Report").size(15).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }))
            .padding([12, 28]).style(primary_button_style).on_press(Message::GenerateReport),
    ].spacing(10);

    let controls = card("Report Setup", column![type_section, period_section, filter_section, gen_section].spacing(20));
    let mut layout = column![controls].spacing(20);
    if state.report_generated { layout = layout.push(view_report_results(state)); }
    layout.into()
}

fn view_report_results(state: &DaycareApp) -> Element<'_, Message> {
    let rows = &state.report_rows;
    let total_mins: i64 = rows.iter().map(|r| r.total_minutes).sum();
    let total_sess: usize = rows.iter().map(|r| r.session_count).sum();
    let period_label = match state.report_type {
        ReportType::Daily => format!(
            "Daily Report -- {}",
            parse_date_value(&state.report_daily_input, "")
                .map(|d| d.format("%B %d, %Y").to_string())
                .unwrap_or_else(|_| state.report_daily_input.clone())
        ),
        ReportType::Weekly => {
            let s = parse_date_value(&state.report_weekly_input, "").ok();
            let e = s.map(|x| x + chrono::Duration::days(6));
            match (s, e) {
                (Some(s), Some(e)) => format!("Weekly Report -- {} to {}", s.format("%b %d"), e.format("%b %d, %Y")),
                _ => "Weekly Report".to_string(),
            }
        }
        ReportType::Monthly => format!("Monthly Report -- {} {}", state.report_month.label, state.report_year),
    };

    let summary_row = row![
        stat_card("Children", rows.len().to_string(), PRIMARY),
        stat_card("Sessions", total_sess.to_string(), SUCCESS),
        stat_card("Total Time", reports::format_minutes(total_mins), PRIMARY),
    ].spacing(16);

    let mut content = column![section_heading(period_label.clone()), summary_row].spacing(14);

    if rows.is_empty() {
        content = content.push(empty_state("No attendance data found for this period."));
    } else {
        content = content.push(table_header_row(&[("Child", 4), ("Sessions", 2), ("Total Time", 3), ("Open Sessions", 2)]));
        for (i, r) in rows.iter().enumerate() {
            content = content.push(table_row(&[
                (r.child_name.clone(), 4),
                (r.session_count.to_string(), 2),
                (reports::format_minutes(r.total_minutes), 3),
                (r.incomplete_sessions.to_string(), 2),
            ], i % 2 == 0, false));
        }
    }

    content = content.push(horizontal_rule(1));
    content = content.push(
        text("Export to CSV").size(16).font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
    );
    content = content.push(
        text("Choose a folder to export to. The filename is set automatically.")
            .size(13).style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) })
    );
    content = content.push(
        row![
            form_field("File path", text_input(
                "Click 'Choose Folder...' to select an output location",
                &state.report_export_path
            ).on_input(Message::ReportExportPathChanged).padding(10).width(Fill)),
            container(
                row![
                    button(text("Choose Folder...").size(14)).padding([10, 16])
                        .style(secondary_button_style).on_press(Message::ExportPickFolder),
                    button(text("Export CSV").size(14)).padding([10, 18])
                        .style(primary_button_style).on_press(Message::ExportCsv),
                ].spacing(8)
            ).padding(iced::Padding { top: 20.0, right: 0.0, bottom: 0.0, left: 0.0 }),
        ].spacing(12).align_y(Alignment::Start),
    );
    if let Some(es) = &state.report_export_status {
        let is_err = es.starts_with("Error") || es.starts_with("Click");
        content = content.push(
            text(es.as_str()).size(14).style(move |_: &Theme| text::Style {
                color: Some(if is_err { DANGER } else { SUCCESS })
            })
        );
    }

    card_element(content.into())
}

fn rtype_btn(label: &'static str, rt: ReportType, current: ReportType) -> Element<'static, Message> {
    button(text(label).size(14)).padding([10, 20])
        .style(if rt == current { primary_button_style } else { secondary_button_style })
        .on_press(Message::ReportTypeSelected(rt)).into()
}

// ─────────────────── Emergency Contacts ─────────────────────────────────────

fn view_emergency_contacts_tab(state: &DaycareApp) -> Element<'_, Message> {
    let q = state.ec_search.to_lowercase();
    let filtered: Vec<_> = state.children.iter().filter(|c| {
        q.is_empty() || c.full_name().to_lowercase().contains(&q)
    }).collect();

    let mut results = column![
        table_header_row(&[("Child", 3), ("Emergency Contact", 3), ("EC Phone", 3), ("Parent", 3), ("Parent Phone", 3)]),
    ].spacing(6);

    if filtered.is_empty() {
        results = results.push(empty_state("No children match your search."));
    } else {
        for (i, c) in filtered.iter().enumerate() {
            let ec_name = if c.emergency_contact_name.is_empty() { "--".to_string() } else { c.emergency_contact_name.clone() };
            let ec_phone = if c.emergency_contact_phone.is_empty() { "--".to_string() } else { c.emergency_contact_phone.clone() };
            results = results.push(table_row(&[
                (c.full_name(), 3), (ec_name, 3), (ec_phone, 3),
                (c.parent.full_name(), 3), (c.parent.phone_number.clone(), 3),
            ], i % 2 == 0, false));
        }
    }

    card("Emergency Contacts", column![
        form_field("Search by child name", text_input("Type a name...", &state.ec_search)
            .on_input(Message::EcSearchChanged).padding(10).width(Fill)),
        results,
    ].spacing(16)).into()
}

// ─────────────────── Settings ───────────────────────────────────────────────

fn view_settings_tab(state: &DaycareApp) -> Element<'_, Message> {
    let mut pw_form = column![
        section_heading("Change Password"),
        text("Enter your current password and choose a new one (minimum 6 characters).")
            .size(14).style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
        form_field("Current Password", text_input("Current password", &state.settings_current_pw)
            .secure(true).on_input(Message::SettingsCurrentPwChanged).padding(10).width(Fill)),
        form_field("New Password", text_input("New password", &state.settings_new_pw)
            .secure(true).on_input(Message::SettingsNewPwChanged).padding(10).width(Fill)),
        form_field("Confirm New Password", text_input("Confirm", &state.settings_confirm_pw)
            .secure(true).on_input(Message::SettingsConfirmPwChanged).padding(10).width(Fill)),
        button(text("Change Password").size(15)).padding([11, 22])
            .style(primary_button_style).on_press(Message::ChangePassword),
    ].spacing(14);

    if let Some(s) = &state.settings_status {
        let c = match s.kind { StatusKind::Success => SUCCESS, StatusKind::Error => DANGER };
        pw_form = pw_form.push(text(s.text.as_str()).size(14)
            .style(move |_: &Theme| text::Style { color: Some(c) }));
    }

    let logout_section = column![
        section_heading("Session"),
        text("Sign out of the application.").size(14)
            .style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
        button(text("Logout").size(15)).padding([11, 22])
            .style(danger_button_style).on_press(Message::Logout),
    ].spacing(12);

    column![card("Account", pw_form), card("Session", logout_section)].spacing(20).into()
}

// ─────────────────── App Logic ──────────────────────────────────────────────

impl DaycareApp {
    fn reset_child_form(&mut self) {
        self.child_form_mode = ChildFormMode::Add;
        self.child_form = ChildForm::default();
    }

    fn reload_children(&mut self) {
        if let Ok(children) = self.store.load_children() { self.children = children; }
        if self.selected_child_id.is_some_and(|id| !self.children.iter().any(|c| c.id == id)) {
            self.selected_child_id = None;
        }
    }

    fn reload_attendance(&mut self) {
        if let Ok(r) = self.store.load_attendance() { self.attendance_records = r; }
    }

    fn child_label(&self, child_id: u32) -> String {
        self.children.iter().find(|c| c.id == child_id)
            .map(|c| c.full_name())
            .unwrap_or_else(|| format!("Child #{child_id}"))
    }

    fn selected_child(&self) -> Option<&ChildRecord> {
        self.selected_child_id.and_then(|id| self.children.iter().find(|c| c.id == id))
    }

    fn set_status(&mut self, msg: StatusMessage) { self.status = Some(msg); }

    fn recent_records_for_child(&self, child_id: u32) -> Vec<&AttendanceRecord> {
        let mut r: Vec<_> = self.attendance_records.iter().filter(|r| r.child_id == child_id).collect();
        r.sort_by_key(|r| Reverse(r.check_in));
        r.truncate(6);
        r
    }

    fn child_options(&self) -> Vec<ChildOption> {
        self.children.iter().map(|c| ChildOption { id: c.id, label: c.full_name() }).collect()
    }

    fn selected_child_option(&self) -> Option<ChildOption> {
        self.selected_child_id.map(|id| ChildOption { id, label: self.child_label(id) })
    }

    fn next_child_id(&self) -> u32 { self.children.iter().map(|c| c.id).max().unwrap_or(0) + 1 }

    fn currently_checked_in_count(&self) -> usize {
        self.children.iter().filter(|c| attendance::is_checked_in(&self.attendance_records, c.id)).count()
    }

    fn status_label(&self, child_id: u32) -> &'static str {
        if attendance::is_checked_in(&self.attendance_records, child_id) { "Checked In" } else { "Checked Out" }
    }

    fn latest_record_for_child(&self, child_id: u32) -> Option<&AttendanceRecord> {
        self.attendance_records.iter().filter(|r| r.child_id == child_id).max_by_key(|r| r.check_in)
    }

    fn last_activity_label(&self, child_id: u32) -> String {
        match self.latest_record_for_child(child_id) {
            Some(r) => match r.check_out {
                Some(co) => format!("Out {}", co.format("%b %d %I:%M %p")),
                None => format!("In {}", r.check_in.format("%b %d %I:%M %p")),
            },
            None => "No sessions".to_string(),
        }
    }

    fn start_editing_child(&mut self, child_id: u32) {
        if let Some(child) = self.children.iter().find(|c| c.id == child_id) {
            self.selected_child_id = Some(child_id);
            self.child_form_mode = ChildFormMode::Edit(child_id);
            self.child_form = ChildForm::from_child(child);
            self.active_tab = MainTab::Children;
            self.children_page = ChildrenPage::AddEdit;
        }
    }

    fn delete_child(&mut self, child_id: u32) {
        match self.store.delete_child(child_id) {
            Ok(()) => {
                let name = self.child_label(child_id);
                self.reload_children();
                self.reload_attendance();
                if self.selected_child_id == Some(child_id) { self.selected_child_id = None; }
                if self.child_form_mode == ChildFormMode::Edit(child_id) { self.reset_child_form(); }
                self.set_status(StatusMessage::success(format!("Record deleted: {name}.")));
            }
            Err(err) => self.set_status(StatusMessage::error(format!("Could not delete: {err}"))),
        }
    }

    fn check_in_child(&mut self, child_id: u32) {
        let ts = Local::now().naive_local();
        match self.store.check_in(child_id, ts) {
            Ok(()) => {
                self.selected_child_id = Some(child_id);
                self.reload_attendance();
                self.attendance_tab = AttendanceTab::CheckOut;
                self.set_status(StatusMessage::success(format!(
                    "{} checked in at {}.",
                    self.child_label(child_id),
                    ts.format("%I:%M %p")
                )));
            }
            Err(_) => self.set_status(StatusMessage::error("This child is already checked in.")),
        }
    }

    fn check_out_child(&mut self, child_id: u32) {
        let ts = Local::now().naive_local();
        match self.store.check_out(child_id, ts) {
            Ok(mins) => {
                self.selected_child_id = Some(child_id);
                self.reload_attendance();
                self.attendance_tab = AttendanceTab::CheckIn;
                self.set_status(StatusMessage::success(format!(
                    "{} checked out. Session: {}.",
                    self.child_label(child_id),
                    reports::format_minutes(mins)
                )));
            }
            Err(_) => self.set_status(StatusMessage::error("This child is not currently checked in.")),
        }
    }

    fn save_child(&mut self) {
        let child_id = match self.child_form_mode {
            ChildFormMode::Add => self.next_child_id(),
            ChildFormMode::Edit(id) => id,
        };
        match self.child_form.build_child(child_id) {
            Ok(child) => match self.child_form_mode {
                ChildFormMode::Add => match self.store.add_child(&child) {
                    Ok(()) => {
                        let name = child.full_name();
                        self.reload_children();
                        self.selected_child_id = Some(child.id);
                        self.reset_child_form();
                        self.children_page = ChildrenPage::Roster;
                        self.set_status(StatusMessage::success(format!("Record added: {name}.")));
                    }
                    Err(err) => self.set_status(StatusMessage::error(format!("Could not add child: {err}"))),
                },
                ChildFormMode::Edit(_) => match self.store.update_child(&child) {
                    Ok(()) => {
                        let name = child.full_name();
                        self.reload_children();
                        self.selected_child_id = Some(child.id);
                        self.reset_child_form();
                        self.children_page = ChildrenPage::Roster;
                        self.set_status(StatusMessage::success(format!("Record updated: {name}.")));
                    }
                    Err(err) => self.set_status(StatusMessage::error(format!("Could not update: {err}"))),
                },
            },
            Err(err) => self.set_status(StatusMessage::error(err)),
        }
    }

    fn generate_report(&mut self) {
        let filter = self.report_child_filter;
        match self.report_type {
            ReportType::Daily => match parse_date_value(&self.report_daily_input, "Enter a valid date (YYYY-MM-DD).") {
                Ok(date) => {
                    self.report_rows = reports::daily_report(&self.children, &self.attendance_records, date, filter);
                    self.report_generated = true;
                }
                Err(e) => self.set_status(StatusMessage::error(e)),
            },
            ReportType::Weekly => match parse_date_value(&self.report_weekly_input, "Enter a valid week start date (YYYY-MM-DD).") {
                Ok(ws) => {
                    self.report_rows = reports::weekly_report(&self.children, &self.attendance_records, ws, filter);
                    self.report_generated = true;
                }
                Err(e) => self.set_status(StatusMessage::error(e)),
            },
            ReportType::Monthly => {
                let year: i32 = match self.report_year.trim().parse() {
                    Ok(y) if y > 1900 && y < 2200 => y,
                    _ => {
                        self.set_status(StatusMessage::error("Enter a valid 4-digit year."));
                        return;
                    }
                };
                self.report_rows = reports::monthly_report(
                    &self.children, &self.attendance_records, year, self.report_month.value, filter,
                );
                self.report_generated = true;
            }
        }
    }

    fn export_csv(&mut self) {
        let path = self.report_export_path.trim().to_string();
        if path.is_empty() {
            self.report_export_status = Some("Click 'Choose Folder...' to set an export location first.".to_string());
            return;
        }
        match reports::export_csv(&self.report_rows, &path) {
            Ok(()) => self.report_export_status = Some(format!("Exported to {path}")),
            Err(err) => self.report_export_status = Some(format!("Error: {err}")),
        }
    }

    fn change_password(&mut self) {
        let current = self.settings_current_pw.clone();
        let new_pw = self.settings_new_pw.trim().to_string();
        let confirm = self.settings_confirm_pw.trim().to_string();
        if !self.store.verify_password(&current) {
            self.settings_status = Some(StatusMessage::error("Current password is incorrect."));
            self.settings_current_pw.clear();
            return;
        }
        if new_pw.len() < 6 {
            self.settings_status = Some(StatusMessage::error("New password must be at least 6 characters."));
            return;
        }
        if new_pw != confirm {
            self.settings_status = Some(StatusMessage::error("Passwords do not match."));
            return;
        }
        match self.store.update_password(&new_pw) {
            Ok(()) => {
                self.settings_current_pw.clear();
                self.settings_new_pw.clear();
                self.settings_confirm_pw.clear();
                self.settings_status = Some(StatusMessage::success("Password changed successfully."));
            }
            Err(err) => self.settings_status = Some(StatusMessage::error(format!("Error: {err}"))),
        }
    }
}

// ─────────────────── UI Helpers ─────────────────────────────────────────────

fn card<'a>(title: &'a str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    card_element(column![
        text(title).size(18).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }),
        horizontal_rule(1),
        content.into(),
    ].spacing(14).into())
}

fn card_element(content: Element<'_, Message>) -> Element<'_, Message> {
    container(content).padding(20).width(Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(CARD)),
            border: iced::Border { width: 1.0, color: BORDER_COLOR, radius: 12.0.into() },
            shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.05), offset: Vector::new(0.0, 2.0), blur_radius: 8.0 },
            ..Default::default()
        }).into()
}

fn stat_card(label: &'static str, value: String, accent: Color) -> Element<'static, Message> {
    container(column![
        text(label).size(12).font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
            .style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
        text(value).size(30).font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
            .style(move |_: &Theme| text::Style { color: Some(accent) }),
    ].spacing(4)).padding(20).width(Length::FillPortion(1))
    .style(|_: &Theme| container::Style {
        background: Some(iced::Background::Color(CARD)),
        border: iced::Border { width: 1.0, color: BORDER_COLOR, radius: 12.0.into() },
        shadow: Shadow { color: Color::from_rgba(0.0, 0.0, 0.0, 0.04), offset: Vector::new(0.0, 2.0), blur_radius: 6.0 },
        ..Default::default()
    }).into()
}

fn form_field<'a>(label: &'a str, input: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(column![
        text(label).size(13).font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
            .style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }),
        input.into(),
    ].spacing(6)).width(Fill).into()
}

fn section_heading(title: impl ToString) -> Element<'static, Message> {
    text(title.to_string()).size(20).font(Font { weight: iced::font::Weight::Bold, ..Default::default() }).into()
}

fn subsection_heading(title: &'static str) -> Element<'static, Message> {
    container(
        text(title).size(14).font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
            .style(|_: &Theme| text::Style { color: Some(PRIMARY) })
    )
    .padding(iced::Padding { top: 8.0, right: 0.0, bottom: 2.0, left: 0.0 })
    .into()
}

fn action_button(label: &'static str, msg: Message) -> Element<'static, Message> {
    button(text(label).size(14)).padding([10, 18]).style(primary_button_style).on_press(msg).into()
}

fn empty_state(msg: &str) -> Element<'_, Message> {
    container(text(msg).size(15).style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) }))
        .padding([24, 0]).width(Fill).into()
}

fn table_header_row<'a>(columns: &'a [(&'a str, u16)]) -> Element<'a, Message> {
    let mut r = Row::new().spacing(12);
    for &(label, portion) in columns {
        r = r.push(container(
            text(label.to_uppercase()).size(11)
                .font(Font { weight: iced::font::Weight::Bold, ..Default::default() })
                .style(|_: &Theme| text::Style { color: Some(TEXT_MUTED) })
        ).width(Length::FillPortion(portion)));
    }
    container(r).padding([6, 14]).width(Fill)
        .style(|_: &Theme| container::Style {
            background: Some(iced::Background::Color(ROW_ALT)),
            border: iced::border::rounded(6.0),
            ..Default::default()
        })
        .into()
}

fn table_row<'a>(columns: &[(String, u16)], is_even: bool, is_selected: bool) -> Element<'a, Message> {
    let mut r = Row::new().spacing(12);
    for (value, portion) in columns {
        r = r.push(container(text(value.clone()).size(14)).width(Length::FillPortion(*portion)));
    }
    container(r).padding([10, 14]).width(Fill).style(move |_: &Theme| table_row_bg(is_even, is_selected)).into()
}

fn table_cell_elem(value: String, portion: u16) -> Element<'static, Message> {
    container(text(value).size(14)).width(Length::FillPortion(portion)).into()
}

fn table_row_bg(is_even: bool, is_selected: bool) -> container::Style {
    if is_selected {
        container::Style { background: Some(iced::Background::Color(SELECTED_ROW)), border: iced::border::rounded(8.0), ..Default::default() }
    } else if is_even {
        container::Style { background: Some(iced::Background::Color(ROW_ALT)), border: iced::border::rounded(8.0), ..Default::default() }
    } else {
        container::Style::default()
    }
}

fn primary_button_style(_: &Theme, status: button::Status) -> button::Style {
    let (bg, tc) = match status {
        button::Status::Hovered | button::Status::Pressed => (PRIMARY_DARK, Color::WHITE),
        button::Status::Disabled => (Color::from_rgb(0.78, 0.85, 0.91), Color::from_rgb(0.65, 0.65, 0.65)),
        _ => (PRIMARY, Color::WHITE),
    };
    button::Style { background: Some(iced::Background::Color(bg)), text_color: tc, border: iced::border::rounded(8.0), ..Default::default() }
}

fn secondary_button_style(_: &Theme, status: button::Status) -> button::Style {
    let (bg, tc) = match status {
        button::Status::Hovered | button::Status::Pressed => (Color::from_rgb(0.88, 0.89, 0.91), Color::from_rgb(0.2, 0.2, 0.2)),
        button::Status::Disabled => (Color::from_rgb(0.94, 0.94, 0.94), Color::from_rgb(0.72, 0.72, 0.72)),
        _ => (Color::from_rgb(0.92, 0.93, 0.95), Color::from_rgb(0.2, 0.2, 0.2)),
    };
    button::Style { background: Some(iced::Background::Color(bg)), text_color: tc, border: iced::border::rounded(8.0), ..Default::default() }
}

fn danger_button_style(_: &Theme, status: button::Status) -> button::Style {
    let (bg, tc) = match status {
        button::Status::Hovered | button::Status::Pressed => (Color::from_rgb(0.75, 0.15, 0.13), Color::WHITE),
        button::Status::Disabled => (Color::from_rgb(0.93, 0.80, 0.80), Color::from_rgb(0.72, 0.72, 0.72)),
        _ => (DANGER, Color::WHITE),
    };
    button::Style { background: Some(iced::Background::Color(bg)), text_color: tc, border: iced::border::rounded(8.0), ..Default::default() }
}

fn success_button_style(_: &Theme, status: button::Status) -> button::Style {
    let (bg, tc) = match status {
        button::Status::Hovered | button::Status::Pressed => (Color::from_rgb(0.2, 0.56, 0.24), Color::WHITE),
        button::Status::Disabled => (Color::from_rgb(0.80, 0.93, 0.81), Color::from_rgb(0.72, 0.72, 0.72)),
        _ => (SUCCESS, Color::WHITE),
    };
    button::Style { background: Some(iced::Background::Color(bg)), text_color: tc, border: iced::border::rounded(8.0), ..Default::default() }
}
