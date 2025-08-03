use secrecy::{ExposeSecret, SecretString};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum AuthorizationKind {
    Basic(Credentials),
    Bearer(String),
}
