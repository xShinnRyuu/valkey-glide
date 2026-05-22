// Copyright Valkey GLIDE Project Contributors - SPDX Identifier: Apache-2.0

//! Retry utilities for cluster operations during failover/recovery.

use super::cluster::ClusterTopology;
use redis::cluster_async::ClusterConnection;
use std::time::Duration;

/// Attempt to get cluster topology with retries.
/// During failover, connections may be in recovery state and CLUSTER NODES
/// can fail transiently. This helper retries with backoff.
pub async fn get_topology_with_retry(
    connection: &mut ClusterConnection,
    max_retries: u32,
    initial_delay: Duration,
) -> ClusterTopology {
    let mut last_err = None;
    for attempt in 0..max_retries {
        match try_get_topology(connection).await {
            Ok(topology) => return topology,
            Err(e) => {
                last_err = Some(e);
                let delay = initial_delay * 2u32.pow(attempt);
                tokio::time::sleep(delay).await;
            }
        }
    }
    panic!(
        "Failed to get CLUSTER NODES after {} retries: {:?}",
        max_retries,
        last_err.unwrap()
    );
}

async fn try_get_topology(
    connection: &mut ClusterConnection,
) -> Result<ClusterTopology, redis::RedisError> {
    use redis::{
        Value,
        cluster_routing::{RoutingInfo, SingleNodeRoutingInfo},
    };

    let nodes_output = connection
        .route_command(
            redis::cmd("CLUSTER").arg("NODES"),
            RoutingInfo::SingleNode(SingleNodeRoutingInfo::Random),
        )
        .await?;

    let nodes_str = match nodes_output {
        Value::BulkString(b) => String::from_utf8_lossy(&b).to_string(),
        Value::VerbatimString { text, .. } => text,
        _ => {
            return Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Unexpected CLUSTER NODES response type",
            )))
        }
    };

    Ok(ClusterTopology::from_nodes_str(&nodes_str))
}
