pub struct FileItem {
    pub name: String,
    pub extension: String,
    pub size: u64,
}

impl FileItem {
    pub fn new(
        name: &str,
        extension: &str,
        size: u64,
    ) -> Self {
        Self {
            name: name.to_string(),
            extension: extension.to_string(),
            size,
        }
    }
}