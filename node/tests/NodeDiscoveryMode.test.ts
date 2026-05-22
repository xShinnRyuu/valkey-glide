/**
 * Copyright Valkey GLIDE Project Contributors - SPDX Identifier: Apache-2.0
 */

import {
    afterAll,
    afterEach,
    beforeAll,
    describe,
    expect,
    it,
} from "@jest/globals";
import { ValkeyCluster } from "../../utils/TestUtils.js";
import { GlideClient, ProtocolVersion, NodeDiscoveryMode } from "../build-ts";
import {
    getClientConfigurationOption,
    getServerVersion,
    parseEndpoints,
} from "./TestUtilities";

describe("NodeDiscoveryMode", () => {
    let cluster: ValkeyCluster;
    let client: GlideClient | undefined;

    beforeAll(async () => {
        const standaloneAddresses: string =
            global.STAND_ALONE_ENDPOINT as string;
        cluster = standaloneAddresses
            ? await ValkeyCluster.initFromExistingCluster(
                  false,
                  parseEndpoints(standaloneAddresses),
                  getServerVersion,
              )
            : await ValkeyCluster.createCluster(false, 1, 1, getServerVersion);
    }, 20000);

    afterEach(async () => {
        client?.close();
        client = undefined;
    });

    afterAll(async () => {
        await cluster.close();
    });

    it.each([ProtocolVersion.RESP2, ProtocolVersion.RESP3])(
        "skip info replication connects and reads_%p",
        async (protocol) => {
            client = await GlideClient.createClient({
                ...getClientConfigurationOption(
                    cluster.getAddresses(),
                    protocol,
                ),
                nodeDiscoveryMode: NodeDiscoveryMode.Static,
            });

            const result = await client.get("nonexistent");
            expect(result).toBeNull();
        },
        10000,
    );

    it.each([ProtocolVersion.RESP2, ProtocolVersion.RESP3])(
        "skip info replication allows writes_%p",
        async (protocol) => {
            // When using Static mode, the client skips INFO REPLICATION and
            // cannot determine primary vs replica. Only pass the first address
            // (primary) to avoid connecting to a replica and getting ReadOnly.
            const addresses = cluster.getAddresses();
            const primaryAddress = [addresses[0]];

            client = await GlideClient.createClient({
                ...getClientConfigurationOption(primaryAddress, protocol),
                nodeDiscoveryMode: NodeDiscoveryMode.Static,
            });

            const key = `skip_write_${Date.now()}`;
            await client.set(key, "value");
            const result = await client.get(key);
            expect(result).toBe("value");
            await client.del([key]);
        },
        10000,
    );

    it.each([ProtocolVersion.RESP2, ProtocolVersion.RESP3])(
        "read only rejects discover replicas_%p",
        async (protocol) => {
            await expect(
                GlideClient.createClient({
                    ...getClientConfigurationOption(
                        cluster.getAddresses(),
                        protocol,
                    ),
                    readOnly: true,
                    nodeDiscoveryMode: NodeDiscoveryMode.DiscoverAll,
                }),
            ).rejects.toThrow();
        },
        10000,
    );
