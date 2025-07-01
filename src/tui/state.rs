use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AppMode {
    Home,
    AddNote,
    ViewNote(Uuid),
    Help,
    Settings,
    AiRewrite { original_note_id: Uuid, rewritten_content: Option<String> },
}

#[derive(Debug, Clone)]
pub enum EditorMode {
    Command,
    Insert,
}

#[derive(Debug, Clone)]
pub enum ActiveField {
    Title,
    Content,
    ApiKey,
    PromptStyle,
    CustomPrompt,
}

#[derive(Debug, Clone)]
pub enum AiState {
    Idle,
    Processing,
    Success,
    Error(String),
}