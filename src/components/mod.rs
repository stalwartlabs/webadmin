pub mod badge;
pub mod card;
pub mod form;
pub mod icon;
pub mod layout;
pub mod list;
pub mod messages;
pub mod report;
pub mod skeleton;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Blue,
    Gray,
    Red,
    Yellow,
    Green,
}
