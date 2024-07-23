use askama::Template;

#[derive(Template, Default)]
#[template(path = "login.html")]
pub struct LoginPage {
    pub username: String,
    pub form: LoginForm,
}

#[derive(Template, Default)]
#[template(path = "login_form.html")]
pub struct LoginForm {
    pub error_message: String,
}
