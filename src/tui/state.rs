use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AppMode {
    Home,
    AddNote,
    ViewNote(Uuid),
    Help,
    Settings,
    AiRewrite { original_note_id: Uuid, rewritten_content: Option<String> },
    Search,
    TagFilter,
    ProjectFilter,
    DeleteConfirm { note_id: Uuid },
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
    Search,
    TagFilter,
    ProjectFilter,
    DeleteOption,
}

#[derive(Debug, Clone)]
pub enum AiState {
    Idle,
    Processing,
    Success,
    Error(String),
}