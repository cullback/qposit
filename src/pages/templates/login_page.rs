use askama::Template;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginPage {
    username: String,
    message: String,
}

impl LoginPage {
    pub fn new(username: String, message: String) -> Self {
        Self { username, message }
    }
}
