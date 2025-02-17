use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
    Frame,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::actions::DynamicAction;
use crate::components::{
    AppLayout, Component, Content, ContentAction, Footer, FooterAction, Header, HeaderAction,
    Modal, ModalAction, ProjectTab, Sidebar, SidebarAction,
};
use crate::persistence::{ProjectData, ProjectUpdate, Storage};

pub enum AppAction {
    Noop,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
enum FocusPosition {
    Header,
    Sidebar,
    Content,
    Footer,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Command,
    TabMode,
    CreateProject,
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
    components: Vec<Box<dyn Component<Action = DynamicAction>>>,
    should_render: bool,
    tick_count: u32,
    current_focus: FocusPosition,
    previous_focus: Option<FocusPosition>,
    mode: Mode,
    current_project: Option<ProjectData>,
    projects: Vec<ProjectData>,
    project_name_buffer: String,
    modal: Option<Modal>,
}

impl App {
    pub fn new(mut storage: Storage) -> Self {
        let (projects, current_project) = Self::init_projects(&mut storage);
        let ui = Self::init_ui(&projects, &current_project);

        let mut app = App {
            storage,
            layout: ui.layout,
            header: ui.header,
            sidebar: ui.sidebar,
            content: ui.content,
            footer: ui.footer,
            components: Vec::new(),
            should_render: true,
            tick_count: 0,
            current_focus: FocusPosition::Sidebar,
            previous_focus: None,
            mode: Mode::Normal,
            current_project,
            projects,
            project_name_buffer: String::new(),
            modal: None,
        };

        // Set initial focus to sidebar
        app.sidebar.focus(true);

        app
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
                id: p.id.clone(),
                name: p.name.clone(),
            })
            .collect();
        let current_project_tab = current_project.clone().map(|p| ProjectTab {
            id: p.id.clone(),
            name: p.name.clone(),
        });
        let layout = AppLayout::new();
        let header = Header::new(project_tabs, current_project_tab);
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
                                id: project_data.id.clone(),
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

    fn unfocus_all_except(&mut self, except: &str) {
        if except != "sidebar" {
            self.sidebar.focus(false);
        }
        if except != "content" {
            self.content.focus(false);
        }
    }

    fn handle_vim_navigation(&mut self, key: KeyCode) {
        let new_focus = match (key, &self.current_focus) {
            // Left
            (KeyCode::Char('h'), FocusPosition::Content) => Some(FocusPosition::Sidebar),
            (KeyCode::Char('h'), FocusPosition::Sidebar) => Some(FocusPosition::Content),

            // Right
            (KeyCode::Char('l'), FocusPosition::Sidebar) => Some(FocusPosition::Content),
            (KeyCode::Char('l'), FocusPosition::Content) => Some(FocusPosition::Sidebar),

            _ => None,
        };

        if let Some(new_focus) = new_focus {
            match new_focus {
                FocusPosition::Header => {
                    self.unfocus_all_except("header");
                    self.header.focus(true);
                }
                FocusPosition::Sidebar => {
                    self.unfocus_all_except("sidebar");
                    self.sidebar.focus(true);
                }
                FocusPosition::Content => {
                    self.unfocus_all_except("content");
                    self.content.focus(true);
                }
                FocusPosition::Footer => {
                    self.unfocus_all_except("footer");
                    self.footer.focus(true);
                }
            }
            self.current_focus = new_focus;
            self.should_render = true;
        }
    }

    fn update_footer_hints(&mut self) {
        let mode_str = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Command => "COMMAND",
            Mode::TabMode => "TAB",
            Mode::CreateProject => "CREATE",
        };

        self.footer.set_mode(mode_str.to_string());
    }

    fn handle_command_mode(&mut self, key: KeyCode) -> AppAction {
        match key {
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
                self.unfocus_all_except("header");
                self.header.focus(true);
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
            // Load the full project data from storage
            if let Ok(Some(project)) = self.storage.load_project(&self.projects[tab_index].id) {
                // Update the current project
                self.current_project = Some(project.clone());

                // Update the sidebar with the new project's requests
                self.sidebar.set_requests(project.requests);

                // Clear the content view since no request is selected
                self.content.clear_request();

                self.should_render = true;
            }
        }
    }

    fn delete_project(&mut self, tab_index: usize) {
        if tab_index < self.projects.len() {
            let project_id = self.projects[tab_index].id.clone();

            // Delete from storage
            if let Err(e) = self.storage.delete_project(&project_id) {
                self.footer
                    .set_status(format!("Failed to delete project: {}", e));
                return;
            }

            // Update projects list
            self.update_projects_list();

            // Update header tabs
            let project_tabs: Vec<ProjectTab> = self
                .projects
                .iter()
                .map(|p| ProjectTab {
                    id: p.id.clone(),
                    name: p.name.clone(),
                })
                .collect();
            let current_project_tab = self.current_project.as_ref().map(|p| ProjectTab {
                id: p.id.clone(),
                name: p.name.clone(),
            });

            // Reset header with new projects
            self.header = Header::new(project_tabs, current_project_tab);
            self.header.focus(true);

            // Update sidebar if current project changed
            if let Some(project) = &self.current_project {
                self.sidebar.set_requests(project.requests.clone());
            } else {
                self.sidebar.set_requests(Vec::new());
            }

            // Clear content view
            self.content.clear_request();

            self.footer
                .set_status("Project deleted successfully".to_string());
            self.should_render = true;
        }
    }

    fn handle_modal_events(&mut self, event: &Event) {
        if let Event::Key(_) | Event::Mouse(_) = event {
            match self
                .modal
                .as_mut()
                .unwrap()
                .tick(Some(event), self.tick_count)
            {
                ModalAction::Close => {
                    self.modal = None;
                    self.should_render = true;
                    // Restore previous focus
                    match self.current_focus {
                        FocusPosition::Header => self.unfocus_all_except("header"),
                        FocusPosition::Sidebar => self.unfocus_all_except("sidebar"),
                        FocusPosition::Content => self.unfocus_all_except("content"),
                        FocusPosition::Footer => self.unfocus_all_except("footer"),
                    }
                }
                ModalAction::Submit(request) => {
                    self.modal = None;
                    self.should_render = true;
                    // Restore previous focus
                    match self.current_focus {
                        FocusPosition::Header => self.unfocus_all_except("header"),
                        FocusPosition::Sidebar => self.unfocus_all_except("sidebar"),
                        FocusPosition::Content => self.unfocus_all_except("content"),
                        FocusPosition::Footer => self.unfocus_all_except("footer"),
                    }
                    if let Some(project) = &mut self.current_project {
                        project.apply_update(ProjectUpdate::AddRequest(request));
                        if let Err(e) = self.storage.save_project(project) {
                            self.footer
                                .set_status(format!("Failed to save project: {}", e));
                        }
                        self.sidebar.set_requests(project.requests.clone());
                    }
                }
                ModalAction::Noop => {
                    self.should_render = true;
                }
            }
        }
    }

    fn handle_tab_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                // Restore previous focus
                if let Some(previous) = self.previous_focus.take() {
                    self.current_focus = previous;
                    match previous {
                        FocusPosition::Header => self.unfocus_all_except("header"),
                        FocusPosition::Sidebar => self.unfocus_all_except("sidebar"),
                        FocusPosition::Content => self.unfocus_all_except("content"),
                        FocusPosition::Footer => self.unfocus_all_except("footer"),
                    }
                }
                self.update_footer_hints();
                self.should_render = true;
            }
            KeyCode::Char(' ') => {
                self.update_footer_hints();
                self.should_render = true;
            }
            _ => {
                // Forward all key events to header
                match self.header.handle_key_event(key.code) {
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
            }
        }
    }

    pub fn tick(&mut self, event: Option<&Event>) -> AppAction {
        self.tick_count = self.tick_count.wrapping_add(1);

        if let Some(event) = event {
            // if modal is active, handle its events first and exclusively
            if let Some(_) = &mut self.modal {
                self.handle_modal_events(event);
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
                    Mode::Normal => match key.code {
                        KeyCode::Char(' ') => {
                            self.mode = Mode::Command;
                            self.update_footer_hints();
                            self.should_render = true;
                            return AppAction::Noop;
                        }
                        KeyCode::Char('h' | 'l') => {
                            self.handle_vim_navigation(key.code);
                            self.update_footer_hints();
                            return AppAction::Noop;
                        }
                        _ => {}
                    },
                    Mode::Command => match key.code {
                        KeyCode::Char(' ') => {
                            self.mode = Mode::Normal;
                            self.update_footer_hints();
                            self.should_render = true;
                            return AppAction::Noop;
                        }
                        KeyCode::Char('q') => return AppAction::Quit,
                        KeyCode::Char('n') => {
                            self.mode = Mode::Normal;
                            self.update_footer_hints();
                            self.should_render = true;
                            return AppAction::Noop;
                        }
                        KeyCode::Char('t') => {
                            self.mode = Mode::TabMode;
                            self.previous_focus = Some(self.current_focus);
                            self.unfocus_all_except("header");
                            self.header.focus(true);
                            self.current_focus = FocusPosition::Header;
                            self.update_footer_hints();
                            self.should_render = true;
                            return AppAction::Noop;
                        }
                        KeyCode::Char('c') => {
                            self.mode = Mode::CreateProject;
                            self.update_footer_hints();
                            self.should_render = true;
                            return AppAction::Noop;
                        }
                        _ => return AppAction::Noop,
                    },
                }
            }

            let header_action = self.header.tick(Some(event), self.tick_count);
            let sidebar_action = self.sidebar.tick(Some(event), self.tick_count);
            let content_action = self.content.tick(Some(event), self.tick_count);
            let footer_action = self.footer.tick(Some(event), self.tick_count);

            match header_action {
                HeaderAction::Focused(true) => {
                    self.unfocus_all_except("header");
                    self.current_focus = FocusPosition::Header;
                    self.update_footer_hints();
                    self.should_render = true;
                }
                HeaderAction::CreateProject => {
                    self.mode = Mode::CreateProject;
                    self.update_footer_hints();
                    self.should_render = true;
                }
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
                SidebarAction::Focused(true) => {
                    self.unfocus_all_except("sidebar");
                    self.current_focus = FocusPosition::Sidebar;
                    self.update_footer_hints();
                    self.should_render = true;
                }
                SidebarAction::Selected(request) => {
                    self.content.set_request(request);
                    self.should_render = true;
                }
                SidebarAction::ProjectUpdate(update) => {
                    if let Some(project) = &mut self.current_project {
                        project.apply_update(update);
                        // Save changes to storage
                        if let Err(e) = self.storage.save_project(project) {
                            self.footer
                                .set_status(format!("Failed to save project: {}", e));
                        }
                        // Update sidebar with new requests
                        self.sidebar.set_requests(project.requests.clone());
                        self.should_render = true;
                    }
                }
                SidebarAction::ShowModal => {
                    self.modal = Some(Modal::new());
                    self.should_render = true;
                }
                _ => {}
            }

            match content_action {
                ContentAction::Focused(true) => {
                    self.unfocus_all_except("content");
                    self.current_focus = FocusPosition::Content;
                    self.update_footer_hints();
                    self.should_render = true;
                }
                ContentAction::ContentUpdated => self.should_render = true,
                _ => {}
            }

            match footer_action {
                FooterAction::Focused(true) => {
                    self.unfocus_all_except("footer");
                    self.current_focus = FocusPosition::Footer;
                    self.update_footer_hints();
                    self.should_render = true;
                }
                FooterAction::StatusUpdated(_) => self.should_render = true,
                _ => {}
            }

            let dynamic_actions: Vec<_> = self
                .components
                .iter_mut()
                .map(|c| c.tick(Some(event), self.tick_count))
                .collect();

            for action in dynamic_actions {
                match action {
                    DynamicAction::Focused(_) => {
                        self.unfocus_all_except("");
                        self.should_render = true;
                    }
                    DynamicAction::Render => self.should_render = true,
                    DynamicAction::Selected(_) => self.should_render = true,
                    DynamicAction::Noop => {}
                }
            }
        }

        AppAction::Noop
    }

    pub fn should_render(&self) -> bool {
        self.should_render
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let (header_area, sidebar_area, content_area, footer_area) =
            self.layout.get_layout_areas(frame.size());

        // Render base components
        self.header.render(frame, header_area);
        self.sidebar.render(frame, sidebar_area);
        self.content.render(frame, content_area);
        self.footer.render(frame, footer_area);

        for component in self.components.iter_mut() {
            component.render(frame, frame.area());
        }

        if let Some(modal) = &mut self.modal {
            modal.render(frame, frame.area());
        }

        self.should_render = false;
    }
}
