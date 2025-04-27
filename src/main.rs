use std::fmt::Debug;
use std::io;
use std::io::BufWriter;
use std::path::PathBuf;
use std::string::ToString;

use clap::{Command, CommandFactory, Parser};
use clap_complete::generate;
use clap_complete::shells::{Bash, Zsh};
use clap_derive::ValueEnum;
use k8s_openapi::api::core::v1::{Namespace, Secret};
use k8s_openapi::serde::de::DeserializeOwned;
use k8s_openapi::{ByteString, Resource};
use kube::api::Api;
use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::{Client, Config};

const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

/// Kubernetes Secrets at your fingertips.
#[derive(Parser)]
#[clap(author, version = VERSION, about, long_about = None)]
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
    Contexts,
    Namespaces,
    Secrets,
    Keys,
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
        return get_completion_script(comp, &mut cmd);
    }

    let kc: Kubeconfig = kubeconfig_from_cli(&cli);

    if let Some((client, ns)) = get_client_ns_from_kubeconfig(kc, &cli).await? {
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
        Some(CompletionHelper::Contexts) => {
            // get contexts from kubeconfig
            let kc: Kubeconfig = kubeconfig_from_cli(cli);
            let contexts: Vec<String> = kc.contexts.iter().map(|c| c.name.clone()).collect();
            for c in contexts {
                println!("{}", c);
            }
        }
        Some(CompletionHelper::Namespaces) => {
            // get namespaces from cluster
            let kc: Kubeconfig = kubeconfig_from_cli(cli);
            if let Some((client, _)) = get_client_ns_from_kubeconfig(kc, cli).await? {
                let namespaces: Api<Namespace> = Api::all(client);
                for_each_resource(namespaces, |n| {
                    println!("{}", n.metadata.name.as_ref().unwrap());
                })
                .await?;
            }
        }
        Some(CompletionHelper::Secrets) => {
            // get secrets from cluster
            let kc: Kubeconfig = kubeconfig_from_cli(cli);
            if let Some((client, ns)) = get_client_ns_from_kubeconfig(kc, cli).await? {
                let secrets: Api<Secret> = Api::namespaced(client, ns.as_str());
                for_each_resource(secrets, |s| {
                    println!("{}", s.metadata.name.as_ref().unwrap());
                })
                .await?;
            }
        }
        Some(CompletionHelper::Keys) => {
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

// Perform an action for each resource in an API
async fn for_each_resource<T: Resource + Clone + DeserializeOwned + Debug>(
    api: Api<T>,
    process: impl Fn(&T),
) -> anyhow::Result<()> {
    api.list(&Default::default())
        .await?
        .iter()
        .for_each(process);
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

async fn get_client_ns_from_kubeconfig(
    kc: Kubeconfig,
    cli: &Cli,
) -> anyhow::Result<Option<(Client, String)>> {
    if let Some(kco) = config_options_for_context(kc, cli.context.clone()) {
        let config = Config::from_kubeconfig(&kco).await.unwrap();
        let client = Client::try_from(config).unwrap();
        let ns = cli
            .namespace
            .clone()
            .unwrap_or(client.default_namespace().to_string());
        Ok(Some((client, ns)))
    } else {
        Ok(None)
    }
}

fn get_completion_script(comp: Shell, cmd: &mut Command) -> anyhow::Result<()> {
    match comp {
        Shell::Bash => generate(Bash, cmd, KSEC, &mut io::stdout()),
        Shell::Zsh => {
            // use writer to write to string:
            let mut bw = BufWriter::new(Vec::new());
            generate(Zsh, cmd, KSEC, &mut bw);
            let mut s = String::from_utf8(bw.buffer().to_vec()).unwrap();
            let zsh_include_sh = String::from("&& ret=0\n")
                + include_str!("zsh.include.sh")
                    // remove lines with #trim or starting with #! from zsh script
                    .lines()
                    .filter(|l| !l.contains("#trim") && !l.starts_with("#!"))
                    .collect::<Vec<&str>>()
                    .join("\n")
                    .as_str()
                + "\n";
            s = s
                .replace("&& ret=0\n", zsh_include_sh.as_str())
                .replace(":CONTEXT: ", ":CONTEXT:->contexts ")
                .replace(":NAMESPACE: ", ":NAMESPACE:->namespaces ")
                .replace(
                    ":secret -- Name of the secret:",
                    ":secret -- Name of the secret:->secrets",
                )
                .replace("::key -- Key in secret:", "::key -- Key in secret:->keys");
            print!("{}", s);
        }
    }
    Ok(())
}
