# datasynth-server

REST, gRPC, and WebSocket server for synthetic data generation.

## Overview

`datasynth-server` provides server-based access to SyntheticData:

- **REST API**: Configuration management and stream control
- **gRPC API**: High-performance streaming generation
- **WebSocket**: Real-time event streaming
- **Production Features**: Authentication, rate limiting, timeouts

## Starting the Server

```bash
cargo run -p datasynth-server -- --port 3000 --worker-threads 4
```

## REST API

### Configuration

```bash
# Get current configuration
curl http://localhost:3000/api/config

# Update configuration
curl -X POST http://localhost:3000/api/config \
  -H "Content-Type: application/json" \
  -d '{"industry": "manufacturing"}'
```

### Stream Control

```bash
# Start generation
curl -X POST http://localhost:3000/api/stream/start

# Pause/Resume
curl -X POST http://localhost:3000/api/stream/pause
curl -X POST http://localhost:3000/api/stream/resume

# Stop
curl -X POST http://localhost:3000/api/stream/stop

# Trigger pattern
curl -X POST http://localhost:3000/api/stream/trigger/month_end
```

### Health Check

```bash
curl http://localhost:3000/health
```

## WebSocket Streaming

Connect to `ws://localhost:3000/ws/events` for real-time events.

## Authentication

Set the `X-API-Key` header for authenticated requests:

```bash
curl -H "X-API-Key: your-api-key" http://localhost:3000/api/config
```

## gRPC

Protocol buffer definitions in `proto/synth.proto`.

```protobuf
service SynthService {
  rpc StreamGenerate(GenerateRequest) returns (stream GenerateResponse);
  rpc GetConfig(Empty) returns (ConfigResponse);
  rpc SetConfig(ConfigRequest) returns (ConfigResponse);
}
```

## Production Features

| Feature | Description |
|---------|-------------|
| Rate Limiting | Sliding window with per-client tracking |
| Request Timeout | Configurable timeout layer |
| Memory Limits | Enforced via memory guard |
| CORS | Configurable cross-origin settings |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
