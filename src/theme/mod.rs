use ratatui::style::Color;

#[derive(Clone)]
pub struct GeneralColors {
    pub bg: Color,
    pub content_bg: Color,
    pub selected_bg: Color,
    pub text: Color,
    pub text_unfocused: Color,
    pub title_focused: Color,
    pub title_unfocused: Color,
}

#[derive(Clone)]
pub struct HttpMethodColors {
    pub get: Color,
    pub post: Color,
    pub put: Color,
    pub delete: Color,
    pub patch: Color,
    pub head: Color,
    pub default: Color,
}

#[derive(Clone)]
pub struct FooterColors {
    pub bg: Color,
    pub border: Color,
    pub mode_normal: Color,
    pub mode_command: Color,
    pub mode_tab: Color,
    pub mode_create: Color,
    pub key_bg: Color,
    pub key_fg: Color,
    pub description: Color,
}

#[derive(Clone)]
pub struct SidebarColors {
    pub bg: Color,
    pub selected_bg: Color,
    pub text: Color,
    pub text_unfocused: Color,
    pub title_focused: Color,
    pub title_unfocused: Color,
}

impl Default for GeneralColors {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(10, 10, 10),
            content_bg: Color::Rgb(16, 18, 23),
            selected_bg: Color::Rgb(30, 30, 30),
            text: Color::White,
            text_unfocused: Color::Gray,
            title_focused: Color::LightBlue,
            title_unfocused: Color::Gray,
        }
    }
}

impl Default for HttpMethodColors {
    fn default() -> Self {
        Self {
            get: Color::Rgb(97, 175, 254),   // Bright blue
            post: Color::Rgb(73, 204, 144),  // Green
            put: Color::Rgb(252, 161, 48),   // Orange
            delete: Color::Rgb(249, 62, 62), // Red
            patch: Color::Rgb(80, 227, 194), // Teal
            head: Color::Rgb(144, 97, 249),  // Purple
            default: Color::Gray,
        }
    }
}

impl Default for FooterColors {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(16, 18, 24),
            border: Color::DarkGray,
            mode_normal: Color::Green,
            mode_command: Color::LightBlue,
            mode_tab: Color::Yellow,
            mode_create: Color::Magenta,
            key_bg: Color::Black,
            key_fg: Color::White,
            description: Color::White,
        }
    }
}

impl Default for SidebarColors {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(10, 10, 10),
            selected_bg: Color::Rgb(30, 30, 30),
            text: Color::White,
            text_unfocused: Color::Gray,
            title_focused: Color::LightBlue,
            title_unfocused: Color::Gray,
        }
    }
}

#[derive(Clone)]
pub struct Theme {
    pub general: GeneralColors,
    pub http_methods: HttpMethodColors,
    pub footer: FooterColors,
    pub sidebar: SidebarColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            general: GeneralColors::default(),
            http_methods: HttpMethodColors::default(),
            footer: FooterColors::default(),
            sidebar: SidebarColors::default(),
        }
    }
}
