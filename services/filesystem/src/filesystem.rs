use std::fs;

pub struct FilesystemService;

impl FilesystemService {

    pub fn new() -> Self {
        Self
    }

    pub fn list_directory(
        &self,
        path: &str,
    ) {

        println!("Listing: {}", path);

        match fs::read_dir(path) {

            Ok(entries) => {

                for entry in entries {

                    match entry {

                        Ok(item) => {

                            println!(
                                "{}",
                                item.file_name().to_string_lossy()
                            );

                        }

                        Err(error) => {

                            println!("Error: {}", error);

                        }

                    }

                }

            }

            Err(error) => {

                println!("Cannot open directory.");

                println!("{}", error);

            }

        }

    }

}