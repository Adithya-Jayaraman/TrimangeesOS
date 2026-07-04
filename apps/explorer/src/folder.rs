pub struct FolderItem {
    pub name: String,
}

impl FolderItem {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}