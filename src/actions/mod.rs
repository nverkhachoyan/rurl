#[derive(Debug)]
pub enum DynamicAction {
    Noop,
    Render,
    Focused(bool),
    Selected(String),
}
