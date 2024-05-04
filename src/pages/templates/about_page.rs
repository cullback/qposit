use askama::Template;

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutPage {
    username: String,
}

impl AboutPage {
    pub const fn new(username: String) -> Self {
        Self { username }
    }
}
