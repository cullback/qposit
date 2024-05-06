use askama::Template;

#[derive(Template)]
#[template(path = "login_form.html")]
pub struct LoginForm {
    pub error_message: String,
}
