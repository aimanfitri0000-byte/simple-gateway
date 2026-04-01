# 🚀 Simple Gateway – API Gateway dengan Rust & Python

[![CI](https://github.com/aimanfitri0000-byte/simple-gateway/actions/workflows/ci.yml/badge.svg)](https://github.com/aimanfitri0000-byte/simple-gateway/actions/workflows/ci.yml)
[![Docker](https://img.shields.io/badge/docker-ghcr.io-blue)](https://github.com/aimanfitri0000-byte/simple-gateway/pkgs/container/simple-gateway-gateway)
[![Rust](https://img.shields.io/badge/rust-1.74-orange)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.11-blue)](https://www.python.org/)

API Gateway yang dibina dengan **Rust (Axum)** dan microservice **Python (FastAPI)**. Sistem ini mengamalkan konsep microservices dengan fitur lengkap seperti authentication, rate limiting, load balancing, service discovery, dan CI/CD.

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🔐 **JWT Authentication** | Token-based authentication dengan bcrypt password hashing |
| ⏱️ **Rate Limiting** | 10 request/minit per IP menggunakan token bucket algorithm |
| ⚖️ **Load Balancing** | Round Robin distribution antara multiple microservice instances |
| 🔍 **Service Discovery** | Dynamic service discovery dengan Consul + health checks |
| 📊 **Monitoring** | Prometheus metrics endpoint (`/metrics`) dan structured logging |
| 🐳 **Containerization** | Docker images di GitHub Container Registry |
| 🔄 **CI/CD** | GitHub Actions: linting, testing, build, docker push |
| 🧪 **Testing** | Unit tests untuk authentication & rate limiter (9 tests) |

---

## 🏗️ Architecture
┌─────────────┐ ┌─────────────────────────────────────────┐
│ Client │────▶│ Rust API Gateway (Axum) │
│ (curl/web) │ │ • JWT Authentication │
└─────────────┘ │ • Rate Limiting │
│ • Load Balancing (Round Robin) │
│ • Service Discovery (Consul) │
│ • Prometheus Metrics │
└─────────────────────────────────────────┘
│
┌────────────────────┼────────────────────┐
▼ ▼ ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ Python Service │ │ Python Service │ │ Consul │
│ (Port 8001) │ │ (Port 8002) │ │ (Port 8500) │
│ /users │ │ /users │ │ Service Registry│
└─────────────────┘ └─────────────────┘ └─────────────────┘

text

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.74+ ([install](https://rustup.rs/))
- **Python** 3.11+
- **Consul** (optional – untuk service discovery)
- **Docker** (optional – untuk container)

### 1. Clone Repository

```bash
git clone https://github.com/aimanfitri0000-byte/simple-gateway.git
cd simple-gateway
2. Run Microservice (Python)
bash
# Install dependencies
pip install -r requirements.txt

# Run instance 1 (port 8001)
python mock_service.py

# Run instance 2 (port 8002) – terminal berasingan
python mock_service2.py
3. Run Gateway (Rust)
bash
# Build and run
cargo run

# Output:
# 🚀 API Gateway running on http://127.0.0.1:3000
# 🔑 Login: POST /login dengan {"username":"alice","password":"password123"}
4. Test the Gateway
bash
# Health check
curl http://localhost:3000/health

# Login – dapat token
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"password123"}'

# Access protected endpoint dengan token
curl -H "Authorization: Bearer <token>" \
  http://localhost:3000/api/users
🐳 Docker
Pull Images from GitHub Container Registry
bash
# Gateway Rust
docker pull ghcr.io/aimanfitri0000-byte/simple-gateway-gateway:latest

# Python Microservice
docker pull ghcr.io/aimanfitri0000-byte/simple-gateway-python:latest
Run with Docker
bash
# Run gateway
docker run -p 3000:3000 ghcr.io/aimanfitri0000-byte/simple-gateway-gateway:latest

# Run microservice
docker run -p 8001:8001 ghcr.io/aimanfitri0000-byte/simple-gateway-python:latest
🧪 Testing
bash
# Run all unit tests
cargo test

# Output:
# running 9 tests
# test auth::tests::test_generate_token ... ok
# test auth::tests::test_verify_valid_token ... ok
# test rate_limiter::tests::test_rate_limiter_allows_requests_within_limit ... ok
# ...
# test result: ok. 9 passed; 0 failed
📊 API Endpoints
Method	Endpoint	Description	Auth Required
GET	/	Welcome message	❌
GET	/health	Health check	❌
POST	/login	Get JWT token	❌
GET	/api/users	Get users list	✅
GET	/metrics	Prometheus metrics	❌
🔧 Environment Variables
Variable	Description	Default
RUST_LOG	Log level (info, debug, warn, error)	info
JWT_SECRET	Secret key for JWT signing	(dev: your-secret-key)
📁 Project Structure
text
simple-gateway/
├── .github/workflows/     # CI/CD pipeline
├── src/
│   ├── main.rs           # Entry point
│   ├── auth/             # JWT authentication
│   ├── middleware.rs     # Auth middleware
│   ├── rate_limiter.rs   # Rate limiting
│   ├── logging.rs        # Request logging
│   ├── metrics.rs        # Prometheus metrics
│   ├── service_registry.rs   # Service registry & load balancing
│   └── consul_discovery.rs   # Consul service discovery
├── k8s/                  # Kubernetes YAML
├── mock_service.py       # Python microservice (port 8001)
├── mock_service2.py      # Python microservice (port 8002)
├── Dockerfile.gateway    # Rust gateway container
├── Dockerfile.python     # Python microservice container
├── requirements.txt      # Python dependencies
├── requirements-dev.txt  # Python dev dependencies
├── .flake8               # Flake8 configuration
├── pyproject.toml        # Black configuration
├── rustfmt.toml          # Rustfmt configuration
├── format.bat            # Auto-format script (Windows)
└── Cargo.toml            # Rust dependencies
🚦 CI/CD Pipeline (GitHub Actions)
Setiap push ke main atau tag v* akan menjalankan:

✅ Lint & Format – Black (Python) + rustfmt + Clippy

✅ Build & Test – cargo build --release + cargo test

✅ Docker Build – Build dan push images ke GHCR

✅ Version Tag – Auto tag latest, v1.0, sha-xxxxx

📦 Version Tags
Tag	Description
latest	Latest build from main branch
v1.0	Stable release
sha-xxxxx	Specific commit build
🛠️ Technologies Used
Category	Technology
Gateway	Rust, Axum, Tokio
Microservice	Python, FastAPI, Uvicorn
Authentication	JWT, bcrypt
Rate Limiting	Governor (token bucket)
Service Discovery	Consul
Observability	Prometheus, Tracing
Testing	Rust unit tests
CI/CD	GitHub Actions
Container	Docker, GHCR
Orchestration	Kubernetes (YAML sedia)
👨‍💻 Author
Aiman Fitri

GitHub: @aimanfitri0000-byte

⭐ Show Your Support
If this project helped you, please give it a ⭐ on GitHub!