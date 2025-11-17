# QuarXNet

Networking and cluster layer for the QuarXTor storage stack.

Responsibilities:

- RPC / API endpoints for clients and internal services
- Node discovery, membership, health
- Replication / sync protocols
- Cluster metadata management

The underlying storage engine lives in [`QuarXTor/QuarXCore`](https://github.com/QuarXTor/QuarXCore).
