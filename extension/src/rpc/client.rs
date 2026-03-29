pub mod pgl_rpc {
    tonic::include_proto!("pgl_rpc");
}

use anyhow::{bail, ensure};
use pgl_rpc::{pgl_remote_client::PglRemoteClient, CardinalityEstimateRequest, ChoosePlanRequest};
use tonic::transport::Channel;

pub struct PglRemoteSyncClient {
    runtime: tokio::runtime::Runtime,
    client: PglRemoteClient<Channel>,
}

impl PglRemoteSyncClient {
    pub fn connect(addr: String) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let client = runtime.block_on(PglRemoteClient::connect(addr))?;
        Ok(Self { runtime, client })
    }

    pub fn choose_plan(&mut self, plans: Vec<String>) -> anyhow::Result<i32> {
        let request = ChoosePlanRequest { plans };
        let response = self.runtime.block_on(self.client.choose_plan(request))?;
        Ok(response.into_inner().chosen_plan_index)
    }

    pub fn cardinality_estimate(&mut self, rel_opts: Vec<String>) -> anyhow::Result<Vec<i64>> {
        let expected_len = rel_opts.len();
        let request = CardinalityEstimateRequest { rel_opts };
        let response = self
            .runtime
            .block_on(self.client.cardinality_estimate(request))?;
        let estimates = response.into_inner().cardinality_estimates;

        ensure!(
            estimates.len() == expected_len,
            "expected {expected_len} estimates, got {}",
            estimates.len()
        );

        if estimates.iter().any(|estimate| *estimate < 0) {
            bail!("cardinality estimates must be non-negative");
        }

        Ok(estimates)
    }
}
