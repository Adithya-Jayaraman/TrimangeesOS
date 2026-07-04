#[derive(Clone)]
pub struct Application {
    pub name: String,
    pub executable: String,
    pub version: String,
    pub category: String,
}

impl Application {

    pub fn new(
        name: &str,
        executable: &str,
        version: &str,
        category: &str,
    ) -> Self {

        Self {
            name: name.to_string(),
            executable: executable.to_string(),
            version: version.to_string(),
            category: category.to_string(),
        }

    }

}