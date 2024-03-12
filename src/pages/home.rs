use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;

use crate::{
    models,
    templates::{base, home, navbar},
    AppState,
};

pub async fn get(state: State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let Ok(markets) = models::market::get_active_markets(&state.database).await else {
        panic!();
    };

    match state.authenticate(jar).await {
        Some(user) => Html(base(
            &navbar::build_with_username(&user.username),
            &home::build(&user.username, &markets),
        ))
        .into_response(),

        None => Html(base(&navbar::build(), &home::build("world", &markets))).into_response(),
    }
}
