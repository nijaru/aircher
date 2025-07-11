use ratatui::style::{Color, Style};

pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub text: Color,
    pub user_message: Color,
    pub assistant_message: Color,
    pub system_message: Color,
    pub tool_message: Color,
    pub input_text: Color,
    pub border: Color,
    pub highlight: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
}

impl Theme {
    pub fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            background: Color::Black,
            text: Color::White,
            user_message: Color::Green,
            assistant_message: Color::Blue,
            system_message: Color::Red,
            tool_message: Color::Yellow,
            input_text: Color::Yellow,
            border: Color::Gray,
            highlight: Color::LightBlue,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
        }
    }

    pub fn dark() -> Self {
        Self {
            primary: Color::Rgb(108, 122, 237),
            secondary: Color::Rgb(74, 85, 104),
            background: Color::Rgb(26, 32, 44),
            text: Color::Rgb(237, 242, 247),
            user_message: Color::Rgb(104, 211, 145),
            assistant_message: Color::Rgb(99, 179, 237),
            system_message: Color::Rgb(245, 101, 101),
            tool_message: Color::Rgb(246, 173, 85),
            input_text: Color::Rgb(237, 137, 54),
            border: Color::Rgb(113, 128, 150),
            highlight: Color::Rgb(144, 205, 244),
            error: Color::Rgb(245, 101, 101),
            success: Color::Rgb(104, 211, 145),
            warning: Color::Rgb(246, 173, 85),
        }
    }

    pub fn get_message_style(&self, role: &crate::providers::MessageRole) -> Style {
        match role {
            crate::providers::MessageRole::User => Style::default().fg(self.user_message),
            crate::providers::MessageRole::Assistant => Style::default().fg(self.assistant_message),
            crate::providers::MessageRole::System => Style::default().fg(self.system_message),
            crate::providers::MessageRole::Tool => Style::default().fg(self.tool_message),
        }
    }
}
