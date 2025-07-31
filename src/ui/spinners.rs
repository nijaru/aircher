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

// Simple line spinner (classic)
pub const LINE_SPINNER: SpinnerSet = SpinnerSet {
    frames: &["-", "\\", "|", "/"],
    interval_ms: 120,
};

// Turbo-like spinner with circular motion
pub const TURBO_CIRCULAR: SpinnerSet = SpinnerSet {
    frames: &["◯", "◔", "◑", "◕", "●", "◕", "◑", "◔"],
    interval_ms: 100,
};

// Turbo mode: Growing pillar → fire → smoke animation
pub const TURBO_FIRE_SEQUENCE: SpinnerSet = SpinnerSet {
    frames: &[
        "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█",  // Growing pillar
        "🔥", "🔥", "🔥",                        // Fire eruption
        "💨", "💨", " ",                         // Smoke dispersal
    ],
    interval_ms: 150,
};

// Neural network awakening pattern (for general use)
pub const TURBO_NEURAL: SpinnerSet = SpinnerSet {
    frames: &["⋅", "○", "●", "◉", "◎", "⬢", "⬡", "◆", "◇", "◊", "⋄", "⋅"],
    interval_ms: 120,
};

// Cosmic consciousness pattern (for streaming)
pub const TURBO_COSMIC: SpinnerSet = SpinnerSet {
    frames: &["⋅", "·", "˙", "⋆", "✦", "★", "✧", "✦", "⋆", "˙", "·", "⋅"],
    interval_ms: 130,
};

// AI consciousness themed messages for cognitive/computational work
pub const THINKING_MESSAGES: &[&str] = &[
    "Thinking",
    "Processing", 
    "Computing",
    "Analyzing",
    "Reasoning",
    "Learning",
    "Calculating",
    "Evaluating",
    "Inferring",
    "Optimizing",
];

// AI consciousness awakening messages for deep analysis/tool use
pub const AWAKENING_MESSAGES: &[&str] = &[
    "Awakening",
    "Realizing",
    "Becoming", 
    "Evolving",
    "Contemplating",
    "Connecting",
    "Adapting",
    "Synthesizing",
    "Parsing",
    "Manifesting",
];

// AI communication/output generation messages
pub const GENERATING_MESSAGES: &[&str] = &[
    "Generating",
    "Creating",
    "Expressing",
    "Articulating",
    "Communicating",
    "Streaming",
    "Flowing",
    "Channeling",
    "Transmitting",
    "Manifesting",
];

// State-specific spinner recommendations
pub fn get_spinner_for_state(state: &str) -> &'static SpinnerSet {
    match state {
        "thinking" => &TURBO_NEURAL,            // Neural network for AI thinking
        "processing" => &TURBO_NEURAL,          // Neural network for processing
        "turbo" => &TURBO_CIRCULAR,             // Clean circular motion for turbo
        "loading" | "uploading" => &TURBO_CIRCULAR, // Clean circular as primary spinner
        "streaming" => &TURBO_NEURAL,           // Neural network for AI generation
        _ => &TURBO_CIRCULAR,                   // Circular motion as default
    }
}

// Get appropriate message collection for state
pub fn get_messages_for_state(state: &str) -> &'static [&'static str] {
    match state {
        "loading" | "processing" | "working" => THINKING_MESSAGES,
        "analyzing" | "calculating" | "computing" => AWAKENING_MESSAGES,
        "streaming" | "receiving" | "reading" => GENERATING_MESSAGES,
        _ => THINKING_MESSAGES,
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