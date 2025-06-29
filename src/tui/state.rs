use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AppMode {
    Home,
    AddNote,
    ViewNote(Uuid),
    Help,
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
}