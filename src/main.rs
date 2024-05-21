use kube::api::Api;
use kube::{Client, Config};
use kube::config::{Kubeconfig, KubeConfigOptions};
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::ByteString;
use clap::{Parser, Command, CommandFactory};
use clap_derive::ValueEnum;
use clap_complete::generate;
use clap_complete::shells::{Bash, Zsh};
use std::io;

/// Quickly get the value of a Kubernetes Secret.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Context
    #[clap(short, long)]
    context: Option<String>,
    /// Namespace
    #[clap(short, long)]
    namespace: Option<String>,
    secret: String,
    key: Option<String>,
    /// Generate a completion script
    #[clap(long)]
    completion: Option<Shell>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Shell {
    Bash,
    Zsh,
}

const KSEC: &'static str = "ksec";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    let mut cmd: Command = Cli::command();
    if let Some(comp) = cli.completion {
        match comp {
            Shell::Bash => generate(Bash, &mut cmd, KSEC, &mut io::stdout()),
            Shell::Zsh => generate(Zsh, &mut cmd, KSEC, &mut io::stdout()),
        }
        return Ok(());
    }

    let kc: Kubeconfig = Kubeconfig::read()?;
    if let Some(kco) = config_options_for_context(kc, cli.context) {
        let config = Config::from_kubeconfig(&kco).await?;
        let ns = cli.namespace.unwrap_or(config.default_namespace.clone());
        let client = Client::try_from(config)?;
        let secrets: Api<Secret> = Api::namespaced(client, ns.as_str());
        let s = secrets.get(&cli.secret).await?;
        if let Some(data) = &s.data {
            if let Some(k) = &cli.key {
                if let Some(v) = data.get(k.as_str()) {
                    return print_value(v);
                }
            } else {
                if let Some(v) = data.values().next() {
                    return print_value(v);
                }
            }
        }
    }

    Ok(())
}

fn config_options_for_context(kc: Kubeconfig, context: Option<String>) -> Option<KubeConfigOptions> {
    let sought_context = context.or(kc.current_context);
    if let Some(cc) = sought_context {
        for nc in kc.contexts {
            if nc.name == cc {
                let cont = nc.context.unwrap();
                return Some(KubeConfigOptions {
                    context: Some(nc.name),
                    cluster: Some(cont.cluster),
                    user: Some(cont.user),
                });
            }
        }
    }
    None
}

fn print_value(bs: &ByteString) -> anyhow::Result<()> {
    let value = String::from_utf8(bs.0.clone())?;
    println!("{}", value);
    Ok(())
}
