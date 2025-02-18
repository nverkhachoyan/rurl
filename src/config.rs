use crate::theme::{FooterColors, GeneralColors, HttpMethodColors, SidebarColors, Theme};
use dirs::home_dir;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub theme: Option<ThemeConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ThemeConfig {
    pub general: Option<GeneralColorsConfig>,
    pub http_methods: Option<HttpMethodColorsConfig>,
    pub footer: Option<FooterColorsConfig>,
    pub sidebar: Option<SidebarColorsConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeneralColorsConfig {
    pub bg: Option<(u8, u8, u8)>,
    pub selected_bg: Option<(u8, u8, u8)>,
    pub text: Option<(u8, u8, u8)>,
    pub text_unfocused: Option<(u8, u8, u8)>,
    pub title_focused: Option<(u8, u8, u8)>,
    pub title_unfocused: Option<(u8, u8, u8)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpMethodColorsConfig {
    pub get: Option<(u8, u8, u8)>,
    pub post: Option<(u8, u8, u8)>,
    pub put: Option<(u8, u8, u8)>,
    pub delete: Option<(u8, u8, u8)>,
    pub patch: Option<(u8, u8, u8)>,
    pub head: Option<(u8, u8, u8)>,
    pub default: Option<(u8, u8, u8)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FooterColorsConfig {
    pub bg: Option<(u8, u8, u8)>,
    pub border: Option<(u8, u8, u8)>,
    pub mode_normal: Option<(u8, u8, u8)>,
    pub mode_command: Option<(u8, u8, u8)>,
    pub mode_tab: Option<(u8, u8, u8)>,
    pub mode_create: Option<(u8, u8, u8)>,
    pub key_bg: Option<(u8, u8, u8)>,
    pub key_fg: Option<(u8, u8, u8)>,
    pub description: Option<(u8, u8, u8)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidebarColorsConfig {
    pub bg: Option<(u8, u8, u8)>,
    pub selected_bg: Option<(u8, u8, u8)>,
    pub text: Option<(u8, u8, u8)>,
    pub text_unfocused: Option<(u8, u8, u8)>,
    pub title_focused: Option<(u8, u8, u8)>,
    pub title_unfocused: Option<(u8, u8, u8)>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if !config_path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Error parsing config file: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("Error reading config file: {}", e);
                Self::default()
            }
        }
    }

    fn get_config_path() -> PathBuf {
        home_dir()
            .expect("Could not find home directory")
            .join(".rurl")
            .join("config.toml")
    }

    pub fn create_theme(&self) -> Theme {
        let mut theme = Theme::default();

        if let Some(theme_config) = &self.theme {
            if let Some(general) = &theme_config.general {
                Self::apply_general_colors(general, &mut theme.general);
            }
            if let Some(http_methods) = &theme_config.http_methods {
                Self::apply_http_method_colors(http_methods, &mut theme.http_methods);
            }
            if let Some(footer) = &theme_config.footer {
                Self::apply_footer_colors(footer, &mut theme.footer);
            }
            if let Some(sidebar) = &theme_config.sidebar {
                Self::apply_sidebar_colors(sidebar, &mut theme.sidebar);
            }
        }

        theme
    }

    fn apply_general_colors(config: &GeneralColorsConfig, colors: &mut GeneralColors) {
        if let Some(bg) = config.bg {
            colors.bg = Color::Rgb(bg.0, bg.1, bg.2);
        }
        if let Some(selected_bg) = config.selected_bg {
            colors.selected_bg = Color::Rgb(selected_bg.0, selected_bg.1, selected_bg.2);
        }
        if let Some(text) = config.text {
            colors.text = Color::Rgb(text.0, text.1, text.2);
        }
        if let Some(text_unfocused) = config.text_unfocused {
            colors.text_unfocused =
                Color::Rgb(text_unfocused.0, text_unfocused.1, text_unfocused.2);
        }
        if let Some(title_focused) = config.title_focused {
            colors.title_focused = Color::Rgb(title_focused.0, title_focused.1, title_focused.2);
        }
        if let Some(title_unfocused) = config.title_unfocused {
            colors.title_unfocused =
                Color::Rgb(title_unfocused.0, title_unfocused.1, title_unfocused.2);
        }
    }

    fn apply_http_method_colors(config: &HttpMethodColorsConfig, colors: &mut HttpMethodColors) {
        if let Some(get) = config.get {
            colors.get = Color::Rgb(get.0, get.1, get.2);
        }
        if let Some(post) = config.post {
            colors.post = Color::Rgb(post.0, post.1, post.2);
        }
        if let Some(put) = config.put {
            colors.put = Color::Rgb(put.0, put.1, put.2);
        }
        if let Some(delete) = config.delete {
            colors.delete = Color::Rgb(delete.0, delete.1, delete.2);
        }
        if let Some(patch) = config.patch {
            colors.patch = Color::Rgb(patch.0, patch.1, patch.2);
        }
        if let Some(head) = config.head {
            colors.head = Color::Rgb(head.0, head.1, head.2);
        }
        if let Some(default) = config.default {
            colors.default = Color::Rgb(default.0, default.1, default.2);
        }
    }

    fn apply_footer_colors(config: &FooterColorsConfig, colors: &mut FooterColors) {
        if let Some(bg) = config.bg {
            colors.bg = Color::Rgb(bg.0, bg.1, bg.2);
        }
        if let Some(border) = config.border {
            colors.border = Color::Rgb(border.0, border.1, border.2);
        }
        if let Some(mode_normal) = config.mode_normal {
            colors.mode_normal = Color::Rgb(mode_normal.0, mode_normal.1, mode_normal.2);
        }
        if let Some(mode_command) = config.mode_command {
            colors.mode_command = Color::Rgb(mode_command.0, mode_command.1, mode_command.2);
        }
        if let Some(mode_tab) = config.mode_tab {
            colors.mode_tab = Color::Rgb(mode_tab.0, mode_tab.1, mode_tab.2);
        }
        if let Some(mode_create) = config.mode_create {
            colors.mode_create = Color::Rgb(mode_create.0, mode_create.1, mode_create.2);
        }
        if let Some(key_bg) = config.key_bg {
            colors.key_bg = Color::Rgb(key_bg.0, key_bg.1, key_bg.2);
        }
        if let Some(key_fg) = config.key_fg {
            colors.key_fg = Color::Rgb(key_fg.0, key_fg.1, key_fg.2);
        }
        if let Some(description) = config.description {
            colors.description = Color::Rgb(description.0, description.1, description.2);
        }
    }

    fn apply_sidebar_colors(config: &SidebarColorsConfig, colors: &mut SidebarColors) {
        if let Some(bg) = config.bg {
            colors.bg = Color::Rgb(bg.0, bg.1, bg.2);
        }
        if let Some(selected_bg) = config.selected_bg {
            colors.selected_bg = Color::Rgb(selected_bg.0, selected_bg.1, selected_bg.2);
        }
        if let Some(text) = config.text {
            colors.text = Color::Rgb(text.0, text.1, text.2);
        }
        if let Some(text_unfocused) = config.text_unfocused {
            colors.text_unfocused =
                Color::Rgb(text_unfocused.0, text_unfocused.1, text_unfocused.2);
        }
        if let Some(title_focused) = config.title_focused {
            colors.title_focused = Color::Rgb(title_focused.0, title_focused.1, title_focused.2);
        }
        if let Some(title_unfocused) = config.title_unfocused {
            colors.title_unfocused =
                Color::Rgb(title_unfocused.0, title_unfocused.1, title_unfocused.2);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { theme: None }
    }
}

pub fn generate_default_config() -> String {
    r#"# RURL Configuration File

[theme]
# All color values are specified as RGB tuples: (red, green, blue)
# Each component ranges from 0-255
# All fields are optional - if not specified, defaults will be used

[theme.general]
bg = [10, 10, 10]                # Background color
selected_bg = [30, 30, 30]       # Selected item background
text = [255, 255, 255]           # Normal text color
text_unfocused = [128, 128, 128] # Unfocused text color
title_focused = [0, 255, 255]    # Focused title color
title_unfocused = [128, 128, 128] # Unfocused title color

[theme.http_methods]
get = [97, 175, 254]    # GET method color
post = [73, 204, 144]   # POST method color
put = [252, 161, 48]    # PUT method color
delete = [249, 62, 62]  # DELETE method color
patch = [80, 227, 194]  # PATCH method color
head = [144, 97, 249]   # HEAD method color
default = [128, 128, 128] # Default method color

[theme.footer]
bg = [16, 18, 24]           # Footer background
border = [64, 64, 64]       # Footer border
mode_normal = [0, 255, 0]   # Normal mode indicator
mode_command = [0, 255, 255] # Command mode indicator
mode_tab = [255, 255, 0]    # Tab mode indicator
mode_create = [255, 0, 255] # Create mode indicator
key_bg = [0, 0, 0]         # Key background
key_fg = [255, 255, 255]   # Key foreground
description = [255, 255, 255] # Command description text

[theme.sidebar]
bg = [10, 10, 10]                # Sidebar background
selected_bg = [30, 30, 30]       # Selected item background
text = [255, 255, 255]           # Normal text
text_unfocused = [128, 128, 128] # Unfocused text
title_focused = [0, 255, 255]    # Focused title
title_unfocused = [128, 128, 128] # Unfocused title"#
        .to_string()
}
