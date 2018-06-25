pub fn get_file_string(s: &str, id: u32) -> String {
    if let Some(pos) = s.rfind('.') {
        let (left, right) = s.split_at(pos);
        format!("{}{:8X}{}", left, id, right)
    } else {
        format!("{}{:8X}", s, id)
    }.replace(|c: char| {
        !c.is_alphanumeric() && c != '.'
    } , "_")
}
