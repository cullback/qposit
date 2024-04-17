pub mod about;
pub mod home;
pub mod login;
pub mod login_form;
pub mod market;
pub mod open_orders;
pub mod positions;
pub mod signup;
pub mod signup_form;

// pub fn format_decimal<T>(value: T, decimals: usize) -> String
// where
//     u64: From<T>,
//     T: std::fmt::Display,
// {
//     let mut value = value.to_string();
//     value.insert(value.len() - decimals, '.');
//     value
// }

pub fn format_decimal(value: &u32, decimals: usize) -> String {
    let x = 1234;
    println!("{:.2}", x);
    let mut value = value.to_string();
    value.insert(value.len() - decimals, '.');
    value
}
