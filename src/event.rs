pub enum AppAction {
    Submit(String), // user pressed Enter with this text
    Quit,           // Ctrl+C / Ctrl+Q
    Redraw, // resize or cosmetic key
    None,   // event consumed, nothing for App to do
}
