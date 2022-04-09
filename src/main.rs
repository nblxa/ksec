use k8s_openapi::api::core::v1::Secret;
use kube::api::Api;
use kube::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let secrets: Api<Secret> = Api::namespaced(client, "ops");
    let s = secrets.get("ops-api-token").await?;
    if let Some(data) = &s.data {
        if let Some(token) = data.get("token") {
            let token = String::from_utf8(token.0.clone())?;
            println!("Token: {:?}", token);
        }
    }

    Ok(())
}
