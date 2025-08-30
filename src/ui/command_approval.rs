use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, BorderType, Clear, List, Paragraph, Widget},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApprovalChoice {
    Yes,
    YesForSession,
    EditFeedback,
    No,
    Abort,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Select,
    Input,
}

struct SelectOption {
    label: &'static str,
    #[allow(dead_code)]
    shortcut: char,
    choice: Option<ApprovalChoice>,
    enters_input_mode: bool,
}

const SELECT_OPTIONS: &[SelectOption] = &[
    SelectOption {
        label: "Yes",
        shortcut: 'y',
        choice: Some(ApprovalChoice::Yes),
        enters_input_mode: false,
    },
    SelectOption {
        label: "Yes, always approve this exact command for this session",
        shortcut: 'a',
        choice: Some(ApprovalChoice::YesForSession),
        enters_input_mode: false,
    },
    SelectOption {
        label: "No, and provide feedback (esc)",
        shortcut: 'e',
        choice: Some(ApprovalChoice::EditFeedback),
        enters_input_mode: true,
    },
];

pub struct CommandApprovalModal {
    visible: bool,
    command: String,
    description: String,
    cwd: Option<String>,
    selected_option: usize,
    mode: Mode,
    input_text: String,
    done: bool,
}

impl CommandApprovalModal {
    pub fn new() -> Self {
        Self {
            visible: false,
            command: String::new(),
            description: String::new(),
            cwd: None,
            selected_option: 0,
            mode: Mode::Select,
            input_text: String::new(),
            done: false,
        }
    }

    pub fn show(&mut self, command: String, description: String) {
        self.show_with_cwd(command, description, None);
    }

    pub fn show_with_cwd(&mut self, command: String, description: String, cwd: Option<String>) {
        self.command = command;
        self.description = description;
        self.cwd = cwd;
        self.selected_option = 0;
        self.mode = Mode::Select;
        self.input_text.clear();
        self.done = false;
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.done = true;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn is_complete(&self) -> bool {
        self.done
    }

    pub fn select_next(&mut self) {
        if self.mode == Mode::Select {
            self.selected_option = (self.selected_option + 1) % SELECT_OPTIONS.len();
        }
    }

    pub fn select_prev(&mut self) {
        if self.mode == Mode::Select {
            if self.selected_option == 0 {
                self.selected_option = SELECT_OPTIONS.len() - 1;
            } else {
                self.selected_option -= 1;
            }
        }
    }

    pub fn handle_key(&mut self, key: char) -> Option<ApprovalChoice> {
        match self.mode {
            Mode::Select => self.handle_select_key(key),
            Mode::Input => self.handle_input_key(key),
        }
    }

    pub fn handle_enter(&mut self) -> Option<ApprovalChoice> {
        match self.mode {
            Mode::Select => {
                let option = &SELECT_OPTIONS[self.selected_option];
                if option.enters_input_mode {
                    self.mode = Mode::Input;
                    None
                } else {
                    self.done = true;
                    option.choice
                }
            }
            Mode::Input => {
                self.done = true;
                Some(ApprovalChoice::No) // Input mode always denies with feedback
            }
        }
    }

    pub fn handle_escape(&mut self) -> Option<ApprovalChoice> {
        match self.mode {
            Mode::Select => {
                self.done = true;
                Some(ApprovalChoice::Abort)
            }
            Mode::Input => {
                // Cancel input mode - return to select
                self.mode = Mode::Select;
                self.input_text.clear();
                None
            }
        }
    }

    pub fn add_char_to_input(&mut self, c: char) {
        if self.mode == Mode::Input {
            self.input_text.push(c);
        }
    }

    pub fn remove_char_from_input(&mut self) {
        if self.mode == Mode::Input {
            self.input_text.pop();
        }
    }

    pub fn get_feedback(&self) -> &str {
        &self.input_text
    }

    pub fn get_command(&self) -> &str {
        &self.command
    }

    /// Legacy method for backward compatibility
    pub fn get_selected(&self) -> ApprovalChoice {
        if self.done {
            // Return the last selected option
            let option = &SELECT_OPTIONS[self.selected_option];
            option.choice.unwrap_or(ApprovalChoice::No)
        } else {
            // Default to first option while selecting
            ApprovalChoice::Yes
        }
    }

    fn handle_select_key(&mut self, key: char) -> Option<ApprovalChoice> {
        match key {
            'y' => {
                self.done = true;
                Some(ApprovalChoice::Yes)
            }
            'a' => {
                self.done = true;
                Some(ApprovalChoice::YesForSession)
            }
            'e' => {
                self.mode = Mode::Input;
                None
            }
            'n' => {
                self.done = true;
                Some(ApprovalChoice::No)
            }
            'q' => {
                self.done = true;
                Some(ApprovalChoice::Abort)
            }
            _ => None,
        }
    }

    fn handle_input_key(&mut self, key: char) -> Option<ApprovalChoice> {
        match key {
            '\n' | '\r' => {
                self.done = true;
                Some(ApprovalChoice::No) // Input mode submits as deny with feedback
            }
            _ => {
                self.input_text.push(key);
                None
            }
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Calculate centered area - adjust for content
        let width = (area.width as f32 * 0.8).min(100.0) as u16;
        let height = self.get_height(&area);
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        
        let popup_area = Rect { x, y, width, height };

        // Clear background
        f.render_widget(Clear, popup_area);

        // Main block with rounded borders
        let outer = Block::default()
            .title("Command approval")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Gray));
        
        let inner = outer.inner(popup_area);
        outer.render(popup_area, f.buffer_mut());

        // Calculate prompt height
        let prompt_height = self.get_confirmation_prompt_height(inner.width);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(prompt_height),
                Constraint::Min(0)
            ])
            .split(inner);

        // Confirmation prompt
        let confirmation_prompt = self.build_confirmation_prompt();
        confirmation_prompt.render(chunks[0], f.buffer_mut());

        // Response area - depends on mode
        let response_lines = match self.mode {
            Mode::Select => {
                SELECT_OPTIONS
                    .iter()
                    .enumerate()
                    .map(|(idx, opt)| {
                        let (prefix, style) = if idx == self.selected_option {
                            ("▶", Style::default().fg(Color::Blue))
                        } else {
                            (" ", Style::default())
                        };
                        Line::styled(format!("  {} {}", prefix, opt.label), style)
                    })
                    .collect()
            }
            Mode::Input => {
                vec![
                    Line::from("Give the model feedback on this command:"),
                    Line::from(format!("> {}", self.input_text)),
                ]
            }
        };

        let response_list = List::new(response_lines);
        response_list.render(chunks[1], f.buffer_mut());
    }

    fn get_height(&self, area: &Rect) -> u16 {
        let confirmation_prompt_height = self.get_confirmation_prompt_height(area.width - 2); // -2 for borders
        let border_lines = 2;

        match self.mode {
            Mode::Select => {
                let num_option_lines = SELECT_OPTIONS.len() as u16;
                confirmation_prompt_height + num_option_lines + border_lines
            }
            Mode::Input => {
                let input_lines = 2; // prompt + input field
                confirmation_prompt_height + input_lines + border_lines
            }
        }
    }

    fn get_confirmation_prompt_height(&self, width: u16) -> u16 {
        // Estimate height based on content - this is a simplified calculation
        let mut lines = 4; // Base lines: title + empty + command + empty
        
        if !self.description.is_empty() {
            lines += 2; // description + empty line
        }
        
        lines += 2; // "Allow command?" + empty line
        
        // Add wrapping estimate for long commands
        if self.command.len() > width as usize {
            lines += (self.command.len() / width as usize) as u16;
        }
        
        lines
    }

    fn build_confirmation_prompt(&self) -> Paragraph {
        let mut contents = vec![];

        // Add command directly like Claude Code - clean and simple
        contents.push(Line::from(format!("  {}", self.command)));
        contents.push(Line::from(""));

        // Add description if provided
        if !self.description.is_empty() {
            contents.push(Line::from(format!("  {}", self.description)));
            contents.push(Line::from(""));
        }

        contents.push(Line::from("Do you want to proceed?"));

        Paragraph::new(contents)
    }
}
