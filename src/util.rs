fn is_empty(s: &str) -> bool {
    s.trim().len() == 0 || s.to_ascii_lowercase() == "null"
}

pub trait IsEmpty<T> {
    fn is_empty(t: T) -> bool;
}

impl IsEmpty<String> for String {
    fn is_empty(t: String) -> bool {
        todo!()
    }
}
