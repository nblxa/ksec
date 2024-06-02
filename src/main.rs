use std::io;
use std::io::BufWriter;
use std::path::PathBuf;
use std::string::ToString;

use clap::{Command, CommandFactory, Parser};
use clap_complete::generate;
use clap_complete::shells::{Bash, Zsh};
use clap_derive::ValueEnum;
use k8s_openapi::api::core::v1::{Namespace, Secret};
use k8s_openapi::ByteString;
use kube::{Client, Config};
use kube::api::Api;
use kube::config::{Kubeconfig, KubeConfigOptions};

/// Kubernetes Secrets at your fingertips.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Kubeconfig file
    #[clap(long, value_hint = clap::ValueHint::FilePath)]
    kubeconfig: Option<String>,
    /// Kubectl context
    #[clap(short, long)]
    context: Option<String>,
    /// Namespace
    #[clap(short, long)]
    namespace: Option<String>,
    /// Name of the secret
    secret: String,
    /// Key in secret
    key: Option<String>,
    /// Generate a completion script
    #[clap(long)]
    completion: Option<Shell>,
    #[clap(long, hide = true)]
    completion_helper: Option<CompletionHelper>,
}

#[derive(Clone, ValueEnum)]
enum Shell {
    Bash,
    Zsh,
}

#[derive(Clone, ValueEnum)]
enum CompletionHelper {
    Context,
    Namespace,
    Secret,
    Key,
}

const KSEC: &str = "ksec";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    let mut cmd: Command = Cli::command();
    if cli.completion_helper.is_some() {
        return completion_handler(&cli).await;
    }

    if let Some(comp) = cli.completion {
        match comp {
            Shell::Bash => generate(Bash, &mut cmd, KSEC, &mut io::stdout()),
            Shell::Zsh => {
                // use writer to write to string:
                let mut bw = BufWriter::new(Vec::new());
                generate(Zsh, &mut cmd, KSEC, &mut bw);
                let mut s = String::from_utf8(bw.buffer().to_vec()).unwrap();
                let zsh_include_sh = include_str!("zsh.include.sh");
                s = s.replace("&& ret=0\n", zsh_include_sh)
                    .replace(":CONTEXT: ", ":CONTEXT:->contexts ")
                    .replace(":NAMESPACE: ", ":NAMESPACE:->namespaces ")
                    .replace(":secret -- Name of the secret:", ":secret -- Name of the secret:->secrets")
                    .replace("::key -- Key in secret:", "::key -- Key in secret:->keys");
                print!("{}", s);
            }
        }
        return Ok(());
    }

    let kc: Kubeconfig = kubeconfig_from_cli(&cli);

    if let Some(kco) = config_options_for_context(kc, cli.context) {
        let config = Config::from_kubeconfig(&kco).await?;
        let ns = cli.namespace.unwrap_or(config.default_namespace.clone());
        let client = Client::try_from(config)?;
        let secrets: Api<Secret> = Api::namespaced(client, ns.as_str());
        let res = secrets.get(&cli.secret).await;
        return match res {
            Ok(s) => print_secret(s, &cli.key),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        };
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
            };
        } else if let Some(v) = data.values().next() {
            return print_value(v);
        }
    }
    Err(anyhow::anyhow!("No data found in secret"))
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

fn expand_tilde(path: &str) -> PathBuf {
    let pathbuf = PathBuf::from(path);
    if pathbuf.starts_with("~") {
        let home = dirs::home_dir().unwrap();
        let pathbuf = pathbuf.strip_prefix("~").unwrap();
        let abspath = home.join(pathbuf).to_str().unwrap().to_string();
        PathBuf::from(abspath)
    } else {
        PathBuf::from(path)
    }
}

async fn completion_handler(cli: &Cli) -> anyhow::Result<()> {
    match cli.completion_helper {
        Some(CompletionHelper::Context) => {
            // get contexts from kubeconfig
            let kc: Kubeconfig = kubeconfig_from_cli(cli);
            let contexts: Vec<String> = kc.contexts.iter().map(|c| c.name.clone()).collect();
            for c in contexts {
                println!("{}", c);
            }
        }
        Some(CompletionHelper::Namespace) => {
            // get namespaces from cluster
            let kc: Kubeconfig = kubeconfig_from_cli(cli);
            if let Some((client, _)) = get_client_ns_from_kubeconfig(kc, cli).await? {
                let namespaces: Api<Namespace> = Api::all(client);
                namespaces.list(&Default::default())
                    .await?
                    .iter()
                    .for_each(|n| {
                        println!("{}", n.metadata.name.as_ref().unwrap());
                    });
            }
        }
        Some(CompletionHelper::Secret) => {
            // get secrets from cluster
            let kc: Kubeconfig = kubeconfig_from_cli(cli);
            if let Some((client, ns)) = get_client_ns_from_kubeconfig(kc, cli).await? {
                let secrets: Api<Secret> = Api::namespaced(client, ns.as_str());
                secrets.list(&Default::default())
                    .await?
                    .iter()
                    .for_each(|s| {
                        println!("{}", s.metadata.name.as_ref().unwrap());
                    });
            }
        }
        Some(CompletionHelper::Key) => {
            // get keys from secret
            let kc: Kubeconfig = kubeconfig_from_cli(cli);
            if let Some((client, ns)) = get_client_ns_from_kubeconfig(kc, cli).await? {
                let secrets: Api<Secret> = Api::namespaced(client, ns.as_str());
                let res = secrets.get(&cli.secret).await;
                if let Ok(secret) = res {
                    if let Some(data) = &secret.data {
                        for k in data.keys() {
                            println!("{}", k);
                        }
                    }
                }
            }
        }
        // default case:
        _ => {}
    }
    Ok(())
}

fn kubeconfig_from_cli(cli: &Cli) -> Kubeconfig {
    match &cli.kubeconfig {
        Some(path) => Kubeconfig::read_from(expand_tilde(path.as_str())).unwrap(),
        None => Kubeconfig::from_env()
            .unwrap()
            .or_else(get_kubeconfig)
            .unwrap(),
    }
}

async fn get_client_ns_from_kubeconfig(kc: Kubeconfig, cli: &Cli) -> anyhow::Result<Option<(Client, String)>> {
    if let Some(kco) = config_options_for_context(kc, cli.context.clone()) {
        let config = Config::from_kubeconfig(&kco).await.unwrap();
        let client = Client::try_from(config).unwrap();
        let ns = cli.namespace
            .clone()
            .unwrap_or(client.default_namespace().to_string());
        Ok(Some((client, ns)))
    } else {
        Ok(None)
    }
}
