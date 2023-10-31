use clap::Parser;
use once_cell::sync::Lazy;

#[derive(clap::Parser, Debug)]
pub struct Config {
    #[clap(long, env, default_value = "8080")]
    pub port: u16,
    #[clap(long, env)]
    pub smtp_host: String,
    #[clap(long, env)]
    pub smtp_username: String,
    #[clap(long, env)]
    pub smtp_password: String,
    #[clap(long, env, default_value = "noreply@fncyber")]
    pub email_from: String,
    #[clap(long, env, default_value = "secret")]
    pub jwt_secret: String,
    #[clap(
        long,
        env,
        default_value = "http://scan-workflow-eventsource-svc.argo-events.svc.cluster.local:8082/"
    )]
    pub scan_webhook_url: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);
