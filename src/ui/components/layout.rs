use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub title_area: Rect,
    pub chat_area: Rect,
    pub input_area: Rect,
    pub status_area: Rect,
}

impl AppLayout {
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title bar
                Constraint::Min(0),    // Chat area
                Constraint::Length(3), // Input box
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        Self {
            title_area: chunks[0],
            chat_area: chunks[1],
            input_area: chunks[2],
            status_area: chunks[3],
        }
    }
}

pub struct TitleLayout {
    pub main_title: Rect,
    pub model_info: Rect,
    pub cost_info: Rect,
}

impl TitleLayout {
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // Main title
                Constraint::Min(0),     // Model info
                Constraint::Length(25), // Cost info
            ])
            .split(area);

        Self {
            main_title: chunks[0],
            model_info: chunks[1],
            cost_info: chunks[2],
        }
    }
}
