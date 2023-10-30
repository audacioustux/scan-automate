use clap::Parser;
use once_cell::sync::Lazy;

#[derive(clap::Parser, Debug)]
pub struct Config {
    #[clap(long, env)]
    pub smtp_host: String,
    #[clap(long, env)]
    pub smtp_username: String,
    #[clap(long, env)]
    pub smtp_password: String,
    #[clap(long, env, default_value = "noreply@localhost")]
    pub email_from: String,
    #[clap(long, env, default_value = "secret")]
    pub jwt_secret: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);
