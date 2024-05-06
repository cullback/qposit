use askama::Template;

#[derive(Template)]
#[template(path = "signup.html")]
pub struct Component {
    username: String,
    username_message: String,
    password_message: String,
}

impl Component {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            username_message: String::new(),
            password_message: String::new(),
        }
    }
}
