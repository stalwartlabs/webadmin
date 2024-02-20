pub mod directory;
pub mod login;
pub mod notfound;

pub fn maybe_plural(items: usize, singular: &str, plural: &str) -> String {
    if items == 1 {
        format!("{} {}", items, singular)
    } else {
        format!("{} {}", items, plural)
    }
}
