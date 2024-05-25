use askama::Template;

#[derive(Template, Default)]
#[template(path = "about.html")]
pub struct AboutPage {
    username: String,
}


impl AboutPage {
    pub fn new(username: String) -> Self {
        Self {
            username: username,
        }
    }
}