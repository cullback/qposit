use askama::Template;

use super::signup_form::SignupForm;

#[derive(Template, Default)]
#[template(path = "signup.html")]
pub struct Component {
    username: String,
    form: SignupForm,
}
