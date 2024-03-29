# Websocket benchmark

The performance evaluation of WebSocket servers is crucial for ensuring optimal real-time communication in various applications. In this report, we aim to objectively benchmark the performance of a WebSocket server implemented in JavaScript using uWebSockets.js. The primary focus of our benchmarking effort is to track WebSocket latency under different conditions, with a specific emphasis on tail latency.

## Test Methodology

### Server Implementation

The WebSocket server is implemented in JavaScript utilizing the uWebSockets.js library. It is configured to listen for incoming messages, deserialize them, compute server latency, and then respond with multiple messages to the client. The amplification rate for server responses is set at 4x the client message.

### Client Implementation

The clients are written in Rust, employing the `tokio-tungstenite` library for WebSocket communication. The client code is designed to run asynchronously using the `tokio` runtime, multiplexed over 10 threads to efficiently handle concurrent connections. Each client establishes a WebSocket connection to the server and sends batches of serialized small JSON messages as binary WebSocket messages at predetermined intervals. To synchronize message sending and minimize network contention, each client introduces a slight delay before sending its batch based on its client ID.

### Test Procedure

1. Establish Connections: n_clients connections are established concurrently between the clients and the server.
2. Synchronization: All clients wait for each other to establish connections before proceeding.
3. Message Transmission: Each client sends several batches of messages at predetermined intervals. The size of the batch and the interval are configurable parameters.
4. Message Processing: Clients asynchronously listen for incoming messages and forward them to a sink task responsible for writing results to disk as a `csv` file.
5. Server Response: Upon receiving messages, the server processes them, calculates server latency, and responds with amplified messages to the clients.

### Environment Setup

To simulate real network conditions, both the client and server are deployed within separate Docker containers. A simulated network outbound delay of 20ms for each container is introduced to mimic realistic network latency.

## Running the test:

```bash

```

## Results Analysis

The benchmarking results primarily focus on WebSocket latency, particularly tail latency, under varying conditions such as message size, batch size, and concurrency levels. The key metrics analyzed include:

**Latency Distribution Over Time**:
By monitoring latency distribution over time, we gain insights into how the WebSocket server's performance evolves throughout the benchmark. This allows us to identify trends, anomalies, and potential performance degradation points. For example, spikes in latency may coincide with periods of increased GC activity or memory pressure, indicating potential bottlenecks or resource contention issues.

**Latency Distribution:**
Analyzing latency distribution helps identify outliers and assess the consistency of WebSocket performance.

**Tail Latency**
Understanding tail latency is crucial for ensuring smooth real-time communication, especially in latency-sensitive applications.

```

```
