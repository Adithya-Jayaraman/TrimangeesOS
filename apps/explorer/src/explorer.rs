use crate::file::FileItem;
use crate::folder::FolderItem;

pub struct Explorer {
    pub current_path: String,
    pub files: Vec<FileItem>,
    pub folders: Vec<FolderItem>,
}

impl Explorer {

    pub fn new() -> Self {
        Self {
            current_path: String::from("/"),
            files: Vec::new(),
            folders: Vec::new(),
        }
    }

    pub fn add_file(
        &mut self,
        name: &str,
        extension: &str,
        size: u64,
    ) {
        self.files.push(
            FileItem::new(
                name,
                extension,
                size,
            ),
        );
    }

    pub fn add_folder(
        &mut self,
        name: &str,
    ) {
        self.folders.push(
            FolderItem::new(name),
        );
    }

    pub fn list_contents(&self) {

        println!("Current Folder: {}", self.current_path);

        println!();

        println!("Folders:");

        for folder in &self.folders {
            println!("📁 {}", folder.name);
        }

        println!();

        println!("Files:");

        for file in &self.files {
            println!(
                "📄 {}.{} ({} bytes)",
                file.name,
                file.extension,
                file.size
            );
        }

    }

}