use sapi::{Client, ClientBuilder, SystemBuilder, auth::Credentials};
use std::str::FromStr;
use url::Url;

pub fn setup() -> Client {
    let system = SystemBuilder::default()
        .name("A4H")
        .server_url(Url::from_str("http://localhost:50000").unwrap())
        .build()
        .unwrap();

    ClientBuilder::default()
        .system(system)
        .language("en")
        .client(001)
        .credentials(Credentials::new("DEVELOPER", "ABAPtr2022#01"))
        .build()
        .unwrap()
}
