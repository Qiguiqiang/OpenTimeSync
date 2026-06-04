# TimeSyncWord

Browser-based high-precision time synchronization tool via NTP-weighted clock.

## Features

- **NTP Multi-Server Sync** - Queries 5 NTP servers with weighted averaging and outlier filtering
- **Server Broadcast Mode** - Server pushes time to all clients every 2 seconds (low load)
- **Periodic RTT Measurement** - Network latency measured every 30 seconds
- **Precision Grading** - S+/S/S-/A/B/C/D grades based on offset stability
- **Timezone Support** - 14 timezones with localStorage persistence
- **Cyberpunk UI** - Dark theme with neon glow effects, responsive design
- **Docker Ready** - Multi-stage Docker build with docker-compose

## Architecture

```
NTP Servers (UDP) → Server (Node.js) → WebSocket Broadcast → Client (Browser)
                                                        ↓
                                              Client RTT Ping (every 30s)
```

- **Server**: Queries NTP servers, calculates accurate UTC time, broadcasts to clients
- **Client**: Receives server time, calculates offset, displays synchronized time
- **Hybrid Mode**: Broadcast for time sync + periodic ping for RTT measurement

## Quick Start

### Prerequisites

- **Node.js** 18+ (recommended: 20 LTS)
- **npm** 9+

### Local Development

```bash
# 1. Clone repository
git clone https://github.com/Qiguiqiang/timeSyncWord.git
cd timeSyncWord

# 2. Install dependencies
npm install

# 3. Start server
npm start

# 4. Open browser
# http://localhost:13013
```

### Environment Variables

Create `.env` file (optional):

```bash
PORT=13013                    # HTTP port
SSL_ENABLED=false             # Enable HTTPS
SSL_PORT=13014                # HTTPS port
SSL_KEY_PATH=./certs/server.key
SSL_CERT_PATH=./certs/server.crt
```

### Generate SSL Certificate (for HTTPS)

```bash
# Windows (PowerShell)
.\scripts\generate-ssl.sh

# Linux/Mac
bash scripts/generate-ssl.sh
```

## Project Structure

```
TimeSyncWord/
├── server/
│   ├── index.js           # Server entry (HTTP + WebSocket)
│   ├── config.js          # NTP servers, sync parameters
│   ├── time-service.js    # NTP multi-server weighted sync
│   └── signaling.js       # WebSocket broadcast + RTT handler
├── public/
│   ├── index.html         # Main page
│   ├── css/style.css      # Cyberpunk dark theme
│   └── js/app.js          # Client sync + timezone + UI
├── Dockerfile             # Multi-stage Docker build
├── docker-compose.yml     # Docker orchestration
└── README.md
```

## Precision Grades

| Grade | Offset Std Dev | Description |
|-------|---------------|-------------|
| S+ | < 2ms | Extremely stable |
| S | < 5ms | Very stable |
| S- | < 10ms | Stable |
| A | < 30ms | Good |
| B | < 50ms | Fair |
| C | < 100ms | Poor |
| D | >= 100ms | Unstable |

## Docker Deployment

### Quick Start

```bash
# Build and run in background
docker-compose up -d

# View logs
docker logs -f timesyncword

# Stop
docker-compose down
```

### Build Image Only

```bash
docker build -t timesyncword .
docker run -d -p 13013:13013 --name timesyncword timesyncword
```

### Custom Configuration

```bash
# Override port
PORT=8080 docker-compose up -d

# With SSL
SSL_ENABLED=true docker-compose up -d
```

## Configuration

Edit `server/config.js`:
- `port`: HTTP port (default 13013)
- `ntpServers`: NTP server list
- `sync.samplesPerServer`: Samples per NTP server
- `sync.resyncInterval`: NTP resync interval (ms)

## Time Synchronization Flow

1. **Server → NTP**: Server queries 5 NTP servers (10 samples each)
2. **Server → Client**: Server broadcasts time every 2 seconds via WebSocket
3. **Client Calculation**: Client calculates offset = serverTime - localTime
4. **Client → Server**: Client pings server every 30 seconds for RTT measurement
5. **Display**: Client shows synchronized time with precision grade

## License

MIT
