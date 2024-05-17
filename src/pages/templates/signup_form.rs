use askama::Template;

#[derive(Template)]
#[template(path = "signup_form.html")]
pub struct SignupForm {
    username: String,
    username_message: String,
    password_message: String,
    invite_message: String,
}

impl SignupForm {
    pub const fn new(
        username: String,
        username_message: String,
        password_message: String,
        invite_message: String,
    ) -> Self {
        Self {
            username,
            username_message,
            password_message,
            invite_message,
        }
    }
}
