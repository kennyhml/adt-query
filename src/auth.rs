use secrecy::{ExposeSecret, SecretString};

pub struct Credentials {
    username: String,
    password: SecretString,
}

impl Credentials {
    pub fn new<T: Into<String>>(username: T, password: T) -> Self {
        Self {
            username: username.into(),
            password: SecretString::from(password.into()),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password.expose_secret()
    }
}

pub enum AuthorizationKind {
    Basic(String),
    Bearer(String),
}
