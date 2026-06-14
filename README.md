# Xeno Channel Simulator (Backend stub service)

The Xeno Channel Simulator is a lightweight, asynchronous microservice built in **Rust** designed to simulate messaging provider outcomes (e.g. WhatsApp, SMS, Email, RCS) and fire webhook callbacks back into the CRM.

This separate service models the full async lifecycle of messaging callbacks, enabling robust testing of thundering-herd scenarios, network timeouts, and status updates under load.

---

## 1. Lifecycle Event Progression

When the CRM initiates a campaign dispatch, it makes a POST request to `/send`. The simulator returns a `202 Accepted` immediately and asynchronously schedules outcomes per communication:

1.  **Queued -> Sent**: Fires a callback event after a random delay (0.5s - 3s).
2.  **Sent -> Delivered / Failed**: Fires callback event after a random delay (1s - 5s):
    *   **Delivered**: 90% probability (derived from config limits).
    *   **Failed**: 10% probability (e.g. absent subscriber, connection reset). Sim terminates here.
3.  **Delivered -> Opened**: Fires callback after a random delay (2s - 8s) with 40% probability.
4.  **Opened -> Clicked**: Fires callback after a random delay (1s - 4s) with 25% probability.

---

## 2. Key Architecture Details

*   **Non-Blocking Concurrency**: Uses `tokio::spawn` to run each recipient simulation flow in an independent lightweight task.
*   **Concurrency Throttling (Semaphore)**: Uses a shared `tokio::sync::Semaphore` configured to cap concurrent outbound webhook posts (default max 50). This prevents exhausting the OS sockets/file descriptors and overwhelming the CRM's connection pool.
*   **Exponential Backoff Retries**: If the CRM returns a non-2xx status (or network timeouts occur), the simulator retries transmitting the webhook callback up to 3 times, waiting `1s`, `2s`, and `4s` progressively.

---

## 3. Technology Stack

*   **Server Framework**: Actix-web 4
*   **Asynchronous Runtime**: Tokio 1.0
*   **HTTP Client**: reqwest (pooled client with keep-alive reuse)
*   **Logging**: env_logger + log

---

## 4. Local Installation & Configuration

### 4.1 Prerequisites
*   Rust toolchain installed (cargo, rustc).

### 4.2 Configuration (`.env` or Environment Variables)
Create a `.env` file in the root of the `backend/` folder:
```env
CRM_RECEIPT_URL=http://localhost:3000/api/receipts
WEBHOOK_SECRET=some-shared-secret
PORT=8080
DELIVERY_SUCCESS_RATE=0.70
OPEN_RATE=0.40
CLICK_RATE=0.25
FAILURE_RATE=0.10
```

### 4.3 Running the Server
To compile and run the backend locally:
```bash
cargo run
```
The server will bind to port `8080` by default.

---

## 5. Containerization & Deployment

### 5.1 Dockerfile
The service uses a multi-stage Docker build to keep image sizes extremely small (~10 MB) and clean of build dependencies:
*   **Build stage**: Compiled against the `musl` target inside `rust:1.78-alpine` to build a fully static Rust binary.
*   **Release stage**: Copies the binary into a bare `alpine:3.19` base image.

### 5.2 Deployment
Deploy to **GCP Cloud Run** or any Docker-compatible hosting platform:
1.  Build the Docker image:
    ```bash
    docker build -t xeno-backend .
    ```
2.  Deploy the image, exposing port `8080`, and configure your environment variables. 
3.  Ensure the container has scale-to-zero settings enabled for optimal serverless pricing.
