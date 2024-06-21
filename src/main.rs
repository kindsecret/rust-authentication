#![allow(non_snake_case)]

use std::thread::Scope;

use dioxus::prelude::*;
use tracing::{info, Level};

#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/dashboard")]
    Dashboard {},
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            h1 { "You have to login with google first!" }
            a { href: "http://127.0.0.1:9797/login/google", "Sign in with Google "}
        }
    }
}

#[component]
fn Dashboard() -> Element {
    rsx! {
        h1 { "Welcome to the Dashboard!!!"}
        a { href: "http://127.0.0.1:9797/logout", "logout"}
    }
}
