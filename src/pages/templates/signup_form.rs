use askama::Template;

#[derive(Template)]
#[template(path = "signup_form.html")]
pub struct SignupForm {
    username: String,
    username_message: String,
    password_message: String,
}

impl SignupForm {
    pub fn build_with_error_message(
        username: String,
        username_message: String,
        password_message: String,
    ) -> Self {
        Self {
            username,
            username_message,
            password_message,
        }
    }
}
