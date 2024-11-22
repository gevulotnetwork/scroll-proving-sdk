use async_trait::async_trait;
use clap::Parser;
use reqwest::Url;

use scroll_proving_sdk::{
    config::{CloudProverConfig, Config},
    prover::{
        proving_service::{
            GetVkRequest, GetVkResponse, ProveRequest, ProveResponse, QueryTaskRequest,
            QueryTaskResponse,
        },
        ProverBuilder, ProvingService,
    },
    utils::init_tracing,
};

use gevulot_scroll_sdk::client::Client;
use gevulot_scroll_sdk::client::ProvingService as GevulotProvingService;

#[derive(Parser, Debug)]
#[clap(disable_version_flag = true)]
struct Args {
    /// Path of config file
    #[arg(long = "config", default_value = "config.json")]
    config_file: String,
}

struct GevulotCloudProver {
    proving_service_client: Client,
}

#[async_trait]
impl ProvingService for GevulotCloudProver {
    fn is_local(&self) -> bool {
        false
    }
    async fn get_vk(&self, req: GetVkRequest) -> GetVkResponse {
        let req = types::s2g_get_vk_request(req);
        let res = self.proving_service_client.get_vk(req).await;
        types::g2s_get_vk_response(res)
    }
    async fn prove(&self, req: ProveRequest) -> ProveResponse {
        let req = types::s2g_prove_request(req);
        let res = self.proving_service_client.prove(req).await;
        types::g2s_prove_response(res)
    }
    async fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse {
        let req = types::s2g_query_task_request(req);
        let res = self.proving_service_client.query_task(req).await;
        types::g2s_query_task_response(res)
    }
}

impl GevulotCloudProver {
    pub fn new(cfg: CloudProverConfig) -> Self {
        let base_url = Url::parse(&cfg.base_url).expect("cannot parse cloud prover base_url");
        let api_key = cfg.api_key.to_owned();
        let client = Client::new(base_url.to_string(), Some(api_key));
        Self {
            proving_service_client: client,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let args = Args::parse();
    let cfg: Config = Config::from_file_and_env(args.config_file)?;
    let cloud_prover = GevulotCloudProver::new(
        cfg.prover
            .cloud
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Missing cloud prover configuration"))?,
    );
    let prover = ProverBuilder::new(cfg)
        .with_proving_service(Box::new(cloud_prover))
        .build()
        .await?;

    prover.run().await;

    Ok(())
}

/// Module for converting between identical Gevulot and Scroll types.
pub mod types {
    use scroll_proving_sdk::prover::proving_service::{
        GetVkRequest, GetVkResponse, ProveRequest, ProveResponse, QueryTaskRequest,
        QueryTaskResponse,
    };

    pub fn s2g_get_vk_request(req: GetVkRequest) -> gevulot_scroll_sdk::GetVkRequest {
        unsafe { std::mem::transmute(req) }
    }

    pub fn s2g_prove_request(req: ProveRequest) -> gevulot_scroll_sdk::ProveRequest {
        unsafe { std::mem::transmute(req) }
    }

    pub fn s2g_query_task_request(req: QueryTaskRequest) -> gevulot_scroll_sdk::QueryTaskRequest {
        unsafe { std::mem::transmute(req) }
    }

    pub fn g2s_get_vk_response(res: gevulot_scroll_sdk::GetVkResponse) -> GetVkResponse {
        unsafe { std::mem::transmute(res) }
    }

    pub fn g2s_prove_response(res: gevulot_scroll_sdk::ProveResponse) -> ProveResponse {
        unsafe { std::mem::transmute(res) }
    }

    pub fn g2s_query_task_response(
        res: gevulot_scroll_sdk::QueryTaskResponse,
    ) -> QueryTaskResponse {
        unsafe { std::mem::transmute(res) }
    }
}
