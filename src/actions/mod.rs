mod actions;

pub use actions::Action;

#[derive(Debug)]
pub enum DynamicAction {
    Noop,
    Render,
    Focused(bool),
    Selected(String),
}
