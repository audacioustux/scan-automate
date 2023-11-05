use clap::Parser;
use once_cell::sync::Lazy;

#[derive(clap::Parser, Debug)]
pub struct Config {
    #[clap(long, env, default_value = "4000")]
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
    #[clap(
        long,
        env,
        default_value = "https://argo-server.argo.svc.cluster.local:2746/"
    )]
    pub argo_workflow_url: String,
    #[clap(long, env)]
    pub argo_workflow_token: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::parse);
