use askama::Template;

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutPage {
    pub username: String,
}
