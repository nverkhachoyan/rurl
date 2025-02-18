use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::components::{
    AppLayout, Component, Content, ContentAction, Footer, Header, HeaderAction, ProjectTab,
    Sidebar, SidebarAction,
};
use crate::persistence::{ProjectData, Storage};
use crate::theme::Theme;

pub enum AppAction {
    Noop,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
enum FocusPosition {
    Header,
    Sidebar,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Command,
    TabMode,
    CreateProject,
    EditRequest,
}

struct UiComponents {
    layout: AppLayout,
    header: Header,
    sidebar: Sidebar,
    content: Content,
    footer: Footer,
}

pub struct App {
    storage: Storage,
    layout: AppLayout,
    header: Header,
    sidebar: Sidebar,
    content: Content,
    footer: Footer,
    should_render: bool,
    tick_count: u32,
    current_focus: FocusPosition,
    previous_focus: Option<FocusPosition>,
    mode: Mode,
    current_project: Option<ProjectData>,
    projects: Vec<ProjectData>,
    project_name_buffer: String,
    theme: Theme,
}

impl App {
    pub fn new(mut storage: Storage, theme: Theme) -> Self {
        let (projects, current_project) = Self::init_projects(&mut storage);
        let ui = Self::init_ui(&projects, &current_project);

        App {
            storage,
            layout: ui.layout,
            header: ui.header,
            sidebar: ui.sidebar,
            content: ui.content,
            footer: ui.footer,
            should_render: true,
            tick_count: 0,
            current_focus: FocusPosition::Sidebar,
            previous_focus: None,
            mode: Mode::Normal,
            current_project,
            projects,
            project_name_buffer: String::new(),
            theme,
        }
    }

    fn init_projects(storage: &mut Storage) -> (Vec<ProjectData>, Option<ProjectData>) {
        let projects = storage.list_projects();
        let first_project = projects.first();
        let current_project = first_project
            .map(|p| storage.load_project(&p.id).unwrap_or(None))
            .flatten();

        (projects, current_project)
    }

    fn init_ui(projects: &Vec<ProjectData>, current_project: &Option<ProjectData>) -> UiComponents {
        let project_tabs: Vec<ProjectTab> = projects
            .iter()
            .map(|p| ProjectTab {
                name: p.name.clone(),
            })
            .collect();
        let layout = AppLayout::new();
        let header = Header::new(project_tabs);
        let mut sidebar = Sidebar::new();

        // Set initial requests if we have a current project
        if let Some(project) = current_project {
            sidebar.set_requests(project.requests.clone());
        }

        let content = Content::new();
        let footer = Footer::new();

        UiComponents {
            layout,
            header,
            sidebar,
            content,
            footer,
        }
    }

    fn update_projects_list(&mut self) {
        self.projects = self.storage.list_projects();
        self.current_project = match self.projects.is_empty() {
            true => None,
            false => self
                .storage
                .load_project(&self.projects[0].id)
                .unwrap_or(None),
        };
    }

    fn handle_project_creation(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if !self.project_name_buffer.is_empty() {
                    let project_data = ProjectData {
                        name: self.project_name_buffer.clone(),
                        id: Uuid::new_v4().to_string(),
                        requests: Vec::new(),
                        environments: Vec::new(),
                        created_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        updated_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                    };
                    let ok = self.storage.save_project(&project_data);
                    match ok {
                        Ok(_) => {
                            self.project_name_buffer.clear();
                            self.current_project =
                                self.storage.load_project(&project_data.id).unwrap_or(None);
                            self.update_projects_list();
                            self.header.add_project(ProjectTab {
                                name: project_data.name.clone(),
                            });
                        }
                        Err(e) => {
                            self.footer
                                .set_status(format!("Failed to save project: {}", e));
                        }
                    }
                }

                self.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                self.project_name_buffer.clear();
                self.mode = Mode::Normal;
            }
            KeyCode::Char(c) => {
                self.project_name_buffer.push(c);
                self.footer
                    .set_status(format!("New project name: {}", self.project_name_buffer));
            }
            KeyCode::Backspace => {
                self.project_name_buffer.pop();
                self.footer
                    .set_status(format!("New project name: {}", self.project_name_buffer));
            }
            _ => {}
        }
    }

    fn update_footer_hints(&mut self) {
        let mode_str = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Command => "COMMAND",
            Mode::TabMode => "TAB",
            Mode::CreateProject => "CREATE",
            Mode::EditRequest => "EDIT",
        };

        self.footer.set_mode(mode_str.to_string());
    }

    fn handle_component_actions(&mut self, event: &Event) {
        match self.mode {
            Mode::Normal => {
                if let SidebarAction::Selected(request) =
                    self.sidebar.tick(Some(event), self.tick_count)
                {
                    self.content.set_request(request);
                    self.should_render = true;
                }

                if let HeaderAction::TabChanged(tab_index) =
                    self.header.tick(Some(event), self.tick_count)
                {
                    self.handle_tab_change(tab_index);
                }
            }
            Mode::EditRequest => match self.content.tick(Some(event), self.tick_count) {
                ContentAction::RequestUpdated(request) => {
                    if let Some(project) = &mut self.current_project {
                        if let Some(existing_request) =
                            project.requests.iter_mut().find(|r| r.name == request.name)
                        {
                            *existing_request = request.clone();
                            if let Err(e) = self.storage.save_project(project) {
                                self.footer
                                    .set_status(format!("Failed to save project: {}", e));
                            }
                            self.sidebar.set_requests(project.requests.clone());
                            self.content.set_request(request);
                        }
                    }
                }
                ContentAction::ContentUpdated => {
                    self.should_render = true;
                }
                _ => {}
            },
            _ => {
                let header_action = self.header.tick(Some(event), self.tick_count);
                let sidebar_action = self.sidebar.tick(Some(event), self.tick_count);
                let content_action = self.content.tick(Some(event), self.tick_count);

                match header_action {
                    HeaderAction::TabChanged(tab_index) => {
                        self.handle_tab_change(tab_index);
                    }
                    HeaderAction::DeleteProject(tab_index) => {
                        self.delete_project(tab_index);
                        self.mode = Mode::Normal;
                        self.update_footer_hints();
                    }
                    _ => {}
                }

                match sidebar_action {
                    SidebarAction::Selected(request) => {
                        self.content.set_request(request);
                        self.should_render = true;
                    }
                    SidebarAction::EditRequest => {
                        self.content.enter_edit_mode();
                        self.mode = Mode::EditRequest;
                        self.should_render = true;
                    }
                    SidebarAction::ProjectUpdate(update) => {
                        if let Some(project) = &mut self.current_project {
                            project.apply_update(update);
                            if let Err(e) = self.storage.save_project(project) {
                                self.footer
                                    .set_status(format!("Failed to save project: {}", e));
                            }
                            self.sidebar.set_requests(project.requests.clone());
                            self.should_render = true;
                        }
                    }
                    _ => {}
                }

                match content_action {
                    ContentAction::ContentUpdated => {
                        self.should_render = true;
                    }
                    ContentAction::RequestUpdated(request) => {
                        if let Some(project) = &mut self.current_project {
                            if let Some(existing_request) =
                                project.requests.iter_mut().find(|r| r.name == request.name)
                            {
                                *existing_request = request.clone();
                                if let Err(e) = self.storage.save_project(project) {
                                    self.footer
                                        .set_status(format!("Failed to save project: {}", e));
                                }
                                self.sidebar.set_requests(project.requests.clone());
                                self.content.set_request(request);
                                self.should_render = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_normal_mode(&mut self, key: KeyCode) -> AppAction {
        match key {
            KeyCode::Char(' ') => {
                self.mode = Mode::Command;
                self.update_footer_hints();
                self.should_render = true;
            }
            KeyCode::Char('e') => {
                self.mode = Mode::EditRequest;
                self.content.enter_edit_mode();
                self.should_render = true;
            }
            _ => {
                let event = Event::Key(KeyEvent::new(key, KeyModifiers::empty()));
                if let SidebarAction::Selected(request) =
                    self.sidebar.tick(Some(&event), self.tick_count)
                {
                    self.content.set_request(request);
                    self.should_render = true;
                }
            }
        }
        AppAction::Noop
    }

    fn handle_command_mode(&mut self, key: KeyCode) -> AppAction {
        match key {
            KeyCode::Char(' ') => {
                self.mode = Mode::Normal;
                self.update_footer_hints();
                self.should_render = true;
                AppAction::Noop
            }
            KeyCode::Char('q') => AppAction::Quit,
            KeyCode::Char('n') => {
                self.mode = Mode::Normal;
                self.update_footer_hints();
                self.should_render = true;
                AppAction::Noop
            }
            KeyCode::Char('t') => {
                self.mode = Mode::TabMode;
                self.previous_focus = Some(self.current_focus);
                self.current_focus = FocusPosition::Header;
                self.update_footer_hints();
                self.should_render = true;
                AppAction::Noop
            }
            KeyCode::Char('c') => {
                self.mode = Mode::CreateProject;
                self.update_footer_hints();
                self.should_render = true;
                AppAction::Noop
            }
            _ => AppAction::Noop,
        }
    }

    fn handle_tab_change(&mut self, tab_index: usize) {
        if tab_index < self.projects.len() {
            if let Ok(Some(project)) = self.storage.load_project(&self.projects[tab_index].id) {
                self.current_project = Some(project.clone());
                self.sidebar.set_requests(project.requests);
                self.content.clear_request();
                self.should_render = true;
            }
        }
    }

    fn delete_project(&mut self, tab_index: usize) {
        if tab_index < self.projects.len() {
            let project_id = self.projects[tab_index].id.clone();

            if let Err(e) = self.storage.delete_project(&project_id) {
                self.footer
                    .set_status(format!("Failed to delete project: {}", e));
                return;
            }

            self.update_projects_list();

            let project_tabs: Vec<ProjectTab> = self
                .projects
                .iter()
                .map(|p| ProjectTab {
                    name: p.name.clone(),
                })
                .collect();

            self.header = Header::new(project_tabs);

            if let Some(project) = &self.current_project {
                self.sidebar.set_requests(project.requests.clone());
            } else {
                self.sidebar.set_requests(Vec::new());
            }

            self.content.clear_request();
            self.footer
                .set_status("Project deleted successfully".to_string());
            self.should_render = true;
        }
    }

    fn handle_tab_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                if let Some(previous) = self.previous_focus.take() {
                    self.current_focus = previous;
                }
                self.update_footer_hints();
                self.should_render = true;
            }
            KeyCode::Char(' ') => {
                self.update_footer_hints();
                self.should_render = true;
            }
            _ => match self.header.handle_key_event(key.code) {
                HeaderAction::TabChanged(tab_index) => {
                    self.handle_tab_change(tab_index);
                }
                HeaderAction::DeleteProject(tab_index) => {
                    self.delete_project(tab_index);
                    self.mode = Mode::Normal;
                    self.update_footer_hints();
                }
                _ => {}
            },
        }
    }

    fn handle_edit_mode(&mut self, key: KeyCode) -> AppAction {
        let event = Event::Key(KeyEvent::new(key, KeyModifiers::empty()));
        match self.content.tick(Some(&event), self.tick_count) {
            ContentAction::ContentUpdated => {
                self.should_render = true;
            }
            ContentAction::RequestUpdated(request) => {
                if let Some(project) = &mut self.current_project {
                    if let Some(existing_request) =
                        project.requests.iter_mut().find(|r| r.name == request.name)
                    {
                        *existing_request = request.clone();
                        if let Err(e) = self.storage.save_project(project) {
                            self.footer
                                .set_status(format!("Failed to save project: {}", e));
                        }
                        self.sidebar.set_requests(project.requests.clone());
                        self.content.set_request(request);
                    }
                }
                self.should_render = true;
            }
            ContentAction::Noop => {
                if key == KeyCode::Esc {
                    self.mode = Mode::Normal;
                }
                self.should_render = true;
            }
        }
        AppAction::Noop
    }

    pub fn tick(&mut self, event: Option<&Event>) -> AppAction {
        self.tick_count = self.tick_count.wrapping_add(1);

        if let Some(event) = event {
            if let Event::Resize(_, _) = event {
                self.should_render = true;
                return AppAction::Noop;
            }

            if let Event::Key(key) = event {
                match self.mode {
                    Mode::CreateProject => {
                        self.handle_project_creation(key.code);
                        self.should_render = true;
                        return AppAction::Noop;
                    }
                    Mode::TabMode => {
                        self.handle_tab_events(*key);
                        return AppAction::Noop;
                    }
                    Mode::Normal => return self.handle_normal_mode(key.code),
                    Mode::Command => return self.handle_command_mode(key.code),
                    Mode::EditRequest => return self.handle_edit_mode(key.code),
                }
            }

            self.handle_component_actions(event);
        }

        AppAction::Noop
    }

    pub fn should_render(&self) -> bool {
        self.should_render
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let (header_area, sidebar_area, content_area, footer_area) =
            self.layout.get_layout_areas(frame.area());

        self.header.render(frame, header_area, &self.theme);
        self.sidebar.render(frame, sidebar_area, &self.theme);
        self.content.render(frame, content_area, &self.theme);
        self.footer.render(frame, footer_area, &self.theme);

        self.should_render = false;
    }
}
