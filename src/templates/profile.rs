use askama::Template;

#[derive(Template)]
#[template(path = "profile.html")]
pub struct Component<'a> {
    username: &'a str,
}


pub fn build() -> String {
    Component { username: "hi" }.render().unwrap()
}
