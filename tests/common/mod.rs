use adt_query::{
    Client, ClientBuilder, ConnectionParameters, HttpConnectionBuilder, auth::Credentials,
};
use std::str::FromStr;
use url::Url;

pub fn setup_test_system_client() -> Client<reqwest::Client> {
    let params = HttpConnectionBuilder::default()
        .hostname(Url::from_str("http://localhost:50000").unwrap())
        .client("001")
        .language("en")
        .build()
        .unwrap();

    ClientBuilder::default()
        .connection_params(ConnectionParameters::Http(params))
        .credentials(Credentials::new("DEVELOPER", "ABAPtr2022#01"))
        .dispatcher(reqwest::Client::new())
        .build()
        .unwrap()
}

pub fn setup_unauthorized_client() -> Client<reqwest::Client> {
    let params = HttpConnectionBuilder::default()
        .hostname(Url::from_str("http://localhost:50000").unwrap())
        .client("001")
        .language("en")
        .build()
        .unwrap();

    ClientBuilder::default()
        .connection_params(ConnectionParameters::Http(params))
        .credentials(Credentials::new("Freddie", "Faulig"))
        .dispatcher(reqwest::Client::new())
        .build()
        .unwrap()
}
