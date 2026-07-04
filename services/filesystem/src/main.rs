mod filesystem;

use filesystem::FilesystemService;

fn main() {

    let fs = FilesystemService::new();

    fs.list_directory(".");

}