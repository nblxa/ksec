use std::io;
use std::path::Path;

use clap::{Command, CommandFactory, Parser};
use clap_complete::generate;
use clap_complete::shells::{Bash, Zsh};
use clap_derive::ValueEnum;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::ByteString;
use kube::api::Api;
use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::{Client, Config};

/// Quickly get the value of a Kubernetes Secret.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Kubeconfig
    #[clap(long)]
    kubeconfig: Option<String>,
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

    let kc: Kubeconfig = match cli.kubeconfig {
        Some(path) => Kubeconfig::read_from(Path::new(&path)).unwrap(),
        None => Kubeconfig::from_env()
            .unwrap()
            .or_else(get_kubeconfig)
            .unwrap(),
    };

    if let Some(kco) = config_options_for_context(kc, cli.context) {
        let config = Config::from_kubeconfig(&kco).await?;
        let ns = cli.namespace.unwrap_or(config.default_namespace.clone());
        let client = Client::try_from(config)?;
        let secrets: Api<Secret> = Api::namespaced(client, ns.as_str());
        let res = secrets.get(&cli.secret).await;
        return match res {
            Ok(s) => print_secret(s, &cli.key),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }
    Ok(())
}

fn print_secret(s: Secret, opt_key: &Option<String>) -> anyhow::Result<()> {
    if let Some(data) = &s.data {
        if let Some(k) = opt_key {
            return if let Some(v) = data.get(k) {
                print_value(v)
            } else {
                Err(anyhow::anyhow!("No data found for key: {}", k))
            }
        } else {
            if let Some(v) = data.values().next() {
                return print_value(v);
            }
        }
    }
    return Err(anyhow::anyhow!("No data found in secret"));
}

fn config_options_for_context(
    kc: Kubeconfig,
    context: Option<String>,
) -> Option<KubeConfigOptions> {
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

fn get_kubeconfig() -> Option<Kubeconfig> {
    Some(Kubeconfig::read().unwrap())
}
