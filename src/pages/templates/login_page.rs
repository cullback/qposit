use askama::Template;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginPage {
    pub username: String,
    pub error_message: String,
}
