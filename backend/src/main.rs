// Rocket framework
#[macro_use]
extern crate rocket;

use anyhow::{Context, Error};
use reqwest::header::AUTHORIZATION;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::request;
use rocket::response::{Debug, Redirect};
use rocket::{get, routes};
use rocket_oauth2::{OAuth2, TokenResponse};
use serde_json::{self, Value};
struct User {
    pub username: String,
}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r request::Request<'_>) -> request::Outcome<User, ()> {
        let cookies = request
            .guard::<&CookieJar<'_>>()
            .await
            .expect("request cookies");

        if let Some(cookie) = cookies.get_private("username") {
            return request::Outcome::Success(User {
                username: cookie.value().to_string(),
            });
        }

        request::Outcome::Forward(Status::Unauthorized)
    }
}

/// User information to be retrieved from the Google People API.
#[derive(serde::Deserialize)]
struct GoogleUserInfo {
    names: Vec<Value>,
}

#[get("/login/google")]
fn google_login(oauth2: OAuth2<GoogleUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect(cookies, &["profile"]).unwrap()
}

#[get("/auth/google")]
async fn google_callback(
    token: TokenResponse<GoogleUserInfo>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Debug<Error>> {
    // Use the token to retrieve the user's Google account information.
    let user_info: GoogleUserInfo = reqwest::Client::builder()
        .build()
        .context("failed to build reqwest client")?
        .get("https://people.googleapis.com/v1/people/me?personFields=names")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .send()
        .await
        .context("failed to complete request")?
        .json()
        .await
        .context("failed to deserialize response")?;

    let real_name = user_info
        .names
        .first()
        .and_then(|n| n.get("displayName"))
        .and_then(|s| s.as_str())
        .unwrap_or("");

    // Set a private cookie with the user's name, and redirect to the home page.
    cookies.add_private(
        Cookie::build(("username", real_name.to_string()))
            .same_site(SameSite::Lax)
            .build(),
    );
    Ok(Redirect::to("/"))
}

#[get("/")]
fn index(user: User) -> String {
    format!("Hi, {}!", user.username)
}

#[get("/", rank = 2)]
fn index_anonymous() -> &'static str {
    "Please login (/login/google)"
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::from("username"));
    Redirect::to("http://127.0.0.1:8080")
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 9797)))
        .mount(
            "/",
            routes![
                index,
                index_anonymous,
                logout,
                google_callback,
                google_login,
            ],
        )
        .attach(OAuth2::<GoogleUserInfo>::fairing("google"))
}
