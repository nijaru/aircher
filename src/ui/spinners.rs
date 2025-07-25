/// Collection of spinner animations for different states
/// Inspired by Claude Code's clean aesthetic

#[derive(Clone)]
pub struct SpinnerSet {
    pub frames: &'static [&'static str],
    pub interval_ms: u64,
}

impl SpinnerSet {
    pub fn get_frame(&self, elapsed_ms: u128) -> &'static str {
        let index = (elapsed_ms / self.interval_ms as u128) as usize % self.frames.len();
        self.frames[index]
    }
}

// Core spinner styles for Aircher
pub const BRAILLE_SPINNER: SpinnerSet = SpinnerSet {
    frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    interval_ms: 80,
};

pub const THINKING_SPINNER: SpinnerSet = SpinnerSet {
    frames: &[
        "⠈⠁", "⠈⠑", "⠈⠱", "⠈⡱", "⢀⡱", "⢄⡱", "⢄⡱", "⢆⡱", "⢎⡱", "⢎⡰", 
        "⢎⡠", "⢎⡀", "⢎⠁", "⠎⠁", "⠊⠁"
    ],
    interval_ms: 80,
};

pub const STAR_PULSE: SpinnerSet = SpinnerSet {
    frames: &["⋆", "✦", "★", "✧", "✦", "⋆"],
    interval_ms: 150,
};

pub const GROWING_PILLAR: SpinnerSet = SpinnerSet {
    frames: &["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂"],
    interval_ms: 120,
};

// AI agent and work-themed action messages for different states
pub const HUSTLING_MESSAGES: &[&str] = &[
    "Hustling",
    "Grinding", 
    "Cranking",
    "Churning",
    "Toiling",
    "Laboring",
    "Plugging away",
    "Working hard",
    "Getting busy",
    "Making progress",
];

pub const SCHLEPPING_MESSAGES: &[&str] = &[
    "Schlepping",
    "Computing",
    "Processing", 
    "Analyzing",
    "Crunching numbers",
    "Parsing data",
    "Evaluating",
    "Synthesizing",
    "Deliberating",
    "Contemplating",
];

pub const STREAMING_MESSAGES: &[&str] = &[
    "Streaming",
    "Flowing",
    "Channeling",
    "Transmitting",
    "Relaying",
    "Piping",
    "Buffering",
    "Fetching",
    "Receiving",
    "Ingesting",
];

// State-specific spinner recommendations
pub fn get_spinner_for_state(state: &str) -> &'static SpinnerSet {
    match state {
        "thinking" => &STAR_PULSE,
        "processing" => &THINKING_SPINNER,
        "turbo" => &GROWING_PILLAR,
        "loading" | "uploading" => &BRAILLE_SPINNER,
        "streaming" => &STAR_PULSE, // Use star pulse for streaming
        _ => &BRAILLE_SPINNER,
    }
}

// Get appropriate message collection for state
pub fn get_messages_for_state(state: &str) -> &'static [&'static str] {
    match state {
        "loading" | "processing" | "working" => HUSTLING_MESSAGES,
        "analyzing" | "calculating" | "computing" => SCHLEPPING_MESSAGES,
        "streaming" | "receiving" | "reading" => STREAMING_MESSAGES,
        _ => HUSTLING_MESSAGES,
    }
}

/// Get a random message from the appropriate collection for a state
pub fn get_random_message_for_state(state: &str) -> &'static str {
    let messages = get_messages_for_state(state);
    let index = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as usize % messages.len();
    messages[index]
}

/// Format a thinking display with star spinner
pub fn format_thinking_display(elapsed_ms: u128) -> String {
    let spinner_frame = STAR_PULSE.get_frame(elapsed_ms);
    format!("{} Thinking...", spinner_frame)
}

/// Format a status display with appropriate spinner and message
pub fn format_status_display(state: &str, elapsed_ms: u128) -> String {
    let spinner = get_spinner_for_state(state);
    let spinner_frame = spinner.get_frame(elapsed_ms);
    let message = get_random_message_for_state(state);
    format!("{} {}...", spinner_frame, message)
}