use crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use std::collections::HashMap;

use crate::actions::DynamicAction;
use crate::components::sidebar::ApiRequest;
use crate::components::{
    Component, Content, ContentAction, Footer, FooterAction, Header, HeaderAction, Layout,
    ProjectTab, Sidebar, SidebarAction,
};
use crate::persistence::{ProjectData, Storage};
pub enum AppAction {
    Noop,
    Quit,
}

#[derive(PartialEq)]
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
    layout: Layout,
    header: Header,
    sidebar: Sidebar,
    content: Content,
    footer: Footer,
}

pub struct App {
    storage: Storage,
    layout: Layout,
    header: Header,
    sidebar: Sidebar,
    content: Content,
    footer: Footer,
    components: Vec<Box<dyn Component<Action = DynamicAction>>>,
    should_render: bool,
    tick_count: u32,
    current_focus: FocusPosition,
    mode: Mode,
    current_project: Option<ProjectData>,
    projects: Vec<ProjectData>,
    project_requests: HashMap<usize, Vec<ApiRequest>>,
    project_name_buffer: String,
}

impl App {
    pub fn new(storage: Storage) -> Self {
        let (projects, current_project) = Self::init_projects(&storage);

        let ui = Self::init_ui(&projects, &current_project);

        App {
            storage,
            layout: ui.layout,
            header: ui.header,
            sidebar: ui.sidebar,
            content: ui.content,
            footer: ui.footer,
            components: Vec::new(),
            should_render: true,
            tick_count: 0,
            current_focus: FocusPosition::Content,
            mode: Mode::Normal,
            current_project,
            projects,
            project_requests: HashMap::new(),
            project_name_buffer: String::new(),
        }
    }

    fn init_projects(storage: &Storage) -> (Vec<ProjectData>, Option<ProjectData>) {
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
        let layout = Layout::new();
        let header = Header::new(project_tabs, current_project_tab);
        let sidebar = Sidebar::new();
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
                        id: self.projects.len().to_string(),
                        requests: Vec::new(),
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
        if except != "header" {
            self.header.focus(false);
        }
        if except != "sidebar" {
            self.sidebar.focus(false);
        }
        if except != "content" {
            self.content.focus(false);
        }
        if except != "footer" {
            self.footer.focus(false);
        }
    }

    fn handle_vim_navigation(&mut self, key: KeyCode) {
        let new_focus = match (key, &self.current_focus) {
            // Up
            (KeyCode::Char('k'), FocusPosition::Content) => Some(FocusPosition::Header),
            (KeyCode::Char('k'), FocusPosition::Sidebar) => Some(FocusPosition::Header),

            // Down
            (KeyCode::Char('j'), FocusPosition::Header) => Some(FocusPosition::Sidebar),
            (KeyCode::Char('j'), FocusPosition::Sidebar) => Some(FocusPosition::Content),

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

        let status_line = self.footer.render_status(mode_str);
        self.footer.set_status(status_line.to_string());
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

    pub fn tick(&mut self, event: Option<&Event>) -> AppAction {
        self.tick_count = self.tick_count.wrapping_add(1);

        if let Some(event) = event {
            if let Event::Key(key) = event {
                match self.mode {
                    Mode::CreateProject => {
                        self.handle_project_creation(key.code);
                        self.should_render = true;
                        return AppAction::Noop;
                    }
                    Mode::TabMode => {
                        match key.code {
                            KeyCode::Esc => {
                                self.mode = Mode::Normal;
                                self.update_footer_hints();
                                self.should_render = true;
                                return AppAction::Noop;
                            }
                            KeyCode::Char(' ') => {
                                self.update_footer_hints();
                                self.should_render = true;
                                return AppAction::Noop;
                            }
                            _ => {
                                // forward key events to header for tab navigation
                                if let HeaderAction::TabChanged(tab_index) =
                                    self.header.handle_key_event(key.code)
                                {
                                    let loaded_project =
                                        self.storage.load_project(&self.projects[tab_index].id);
                                    match loaded_project {
                                        Ok(Some(project)) => {
                                            self.current_project = Some(project);
                                        }
                                        _ => {}
                                    }

                                    if let Some(requests) = self.project_requests.get(&tab_index) {
                                        self.sidebar.set_requests(requests.clone());
                                    } else {
                                        self.sidebar.clear_requests();
                                    }
                                    self.should_render = true;
                                }
                                return AppAction::Noop;
                            }
                        }
                    }
                    Mode::Normal => match key.code {
                        KeyCode::Char(' ') => {
                            self.mode = Mode::Command;
                            self.update_footer_hints();
                            self.should_render = true;
                            return AppAction::Noop;
                        }
                        KeyCode::Char('h' | 'j' | 'k' | 'l') => {
                            self.handle_vim_navigation(key.code);
                            self.update_footer_hints();
                            return AppAction::Noop;
                        }
                        _ => {}
                    },
                    Mode::Command => {
                        return self.handle_command_mode(key.code);
                    }
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
                    if let Some(requests) = self.project_requests.get(&tab_index) {
                        self.sidebar.set_requests(requests.clone());
                    } else {
                        self.sidebar.clear_requests();
                    }
                    self.should_render = true;
                }
                HeaderAction::Render => self.should_render = true,
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
                SidebarAction::Render => self.should_render = true,
                _ => {}
            }

            match content_action {
                ContentAction::Focused(true) => {
                    self.unfocus_all_except("content");
                    self.current_focus = FocusPosition::Content;
                    self.update_footer_hints();
                    self.should_render = true;
                }
                ContentAction::Render => self.should_render = true,
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
                FooterAction::Render => self.should_render = true,
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
            self.layout.get_layout_areas(frame.area());

        self.header.render(frame, header_area);
        self.sidebar.render(frame, sidebar_area);
        self.content.render(frame, content_area);
        self.footer.render(frame, footer_area);

        for component in self.components.iter_mut() {
            component.render(frame, frame.area());
        }

        self.should_render = false;
    }
}
