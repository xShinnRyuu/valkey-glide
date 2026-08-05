// Copyright Valkey GLIDE Project Contributors - SPDX Identifier: Apache-2.0

mod constants;
mod utilities;

#[macro_export]
/// Compare `$expected` with `$actual`. This macro, will exit the test process
/// if the assertion fails. Unlike `assert_eq!` - this also works in tasks
macro_rules! async_assert_eq {
    ($expected:expr, $actual:expr) => {{
        if $actual != $expected {
            println!(
                "{}:{}: Expected: {:?} != Actual: {:?}",
                file!(),
                line!(),
                $actual,
                $expected
            );
            std::process::exit(1);
        }
    }};
}

#[cfg(test)]
pub(crate) mod shared_client_tests {
    use glide_core::Telemetry;
    use redis::{cluster_topology::get_slot, cmd};
    use std::collections::HashMap;

    use super::*;
    use glide_core::client::{Client, DEFAULT_RESPONSE_TIMEOUT};
    use glide_core::connection_request::ProtocolVersion;
    use redis::cluster_routing::{SingleNodeRoutingInfo, SlotAddr};
    use redis::{
        FromRedisValue, InfoDict, Pipeline, PipelineRetryStrategy, RedisConnectionInfo, Value,
        cluster_routing::{MultipleNodeRoutingInfo, Route, RoutingInfo},
    };
    use rstest::rstest;
    use utilities::BackingServer;
    use utilities::cluster::*;
    use utilities::*;
