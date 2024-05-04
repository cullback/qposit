use askama::Template;

#[derive(Template)]
#[template(path = "login_form.html")]
pub struct LoginForm {
    message: String,
}

impl LoginForm {
    pub const fn new(message: String) -> Self {
        Self { message }
    }
}
