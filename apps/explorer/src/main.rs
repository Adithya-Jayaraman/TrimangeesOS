mod explorer;
mod file;
mod folder;

use explorer::Explorer;

fn main() {

    let mut explorer = Explorer::new();

    explorer.add_folder("Documents");
    explorer.add_folder("Downloads");

    explorer.add_file(
        "hello",
        "txt",
        2048,
    );

    explorer.add_file(
        "photo",
        "png",
        409600,
    );

    explorer.list_contents();

}