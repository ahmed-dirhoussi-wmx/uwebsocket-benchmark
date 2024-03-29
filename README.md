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

# Running the benchmarks:

## Generate experiment data
The full benchmark suite runs using a single `docker-compose` file. To run the benchmark:

1. Choose a Websocket client by default we use the Rust Websocket client in `clients/wsrust_client`.
2. Choose a Websocket server implementation in `servers`. By default, we use `uwebsocket-js`.
    ```yaml
    services:
        wsclient:
            container_name: rust_client
            build: ./clients/wsrust_client/
            // ...
        wsserver:
            container_name: uwcpp_server
            build: ./servers/uwsjs-server
            // ...
    ```

3. Copy the `.env.example` to `.env` and set the results directory name for saving the experiments' results and the resource limits accordingly
    ```bash
    # Define the result data directory 
    RESULTS_VOLUME=results-uwsjs

    # Define CPU limit
    SERVER_CPU_LIMIT=1
    CLIENT_CPU_LIMIT=10
    ```
4. Change test parameters in `clients/wsrust_client/entrypoint.sh`
    ```bash
    nclients= (1000 3000 5000 10000)
    batch_sizes=(1)
    waits=(1000)
    ```

5. Run the tests using `docker compose up`

## Generate plots and aggregate metrics
To generate the latency plots and aggregate metrics, a separate python scripts in `wrangler/generate_results.py` is provided: 

1. Go to the wrangler directory: `cd wrangler`Create a Python virtual env (using your favorite tool) and activate it. For example using `uv venv && source .venv/bin/activate`
3. Install dependencies : `uv pip install -r requirements.txt`
4. Generate plots and results : `python generate_results.py`. The results are saved in `wrangler/{plots,markdowns}`

## Results Analysis

The benchmarking results primarily focus on WebSocket latency, particularly tail latency, under varying conditions such as message size, batch size, and concurrency levels. The key metrics analyzed include:

**Latency Distribution Over Time**:
By monitoring latency distribution over time, we gain insights into how the WebSocket server's performance evolves throughout the benchmark. This allows us to identify trends, anomalies, and potential performance degradation points. For example, spikes in latency may coincide with periods of increased GC activity or memory pressure, indicating potential bottlenecks or resource contention issues.

**Latency Distribution:**
Analyzing latency distribution helps identify outliers and assess the consistency of WebSocket performance.

**Tail Latency**
Understanding tail latency is crucial for ensuring smooth real-time communication, especially in latency-sensitive applications.

## Results
Running the benchmark for `uWebsocket.js` server using `uwsrust_client` for `nclients=(1000 3000 5000 10000)` with client throughput of `1msg/s` :


**Server Latency**:  time for message to get to server.
|              |   1000 clients |   3000 clients |   5000 clients |   10000 clients |
|:-------------|---------------:|---------------:|---------------:|----------------:|
| mean_latency |             27 |             56 |            115 |             355 |
| max_latency  |            243 |            447 |           3179 |            3909 |
| min_latency  |             20 |             20 |             20 |              20 |
| p50_latency  |             21 |             41 |             80 |             276 |
| p90_latency  |             33 |             86 |            283 |             810 |
| p99_latency  |            135 |            286 |            308 |            1832 |

**RoundTrip Latency**: Time for a message to get to server and back to client
|              |   1000 clients |   3000 clients |   5000 clients |   10000 clients |
|:-------------|---------------:|---------------:|---------------:|----------------:|
| mean_latency |             48 |             77 |            142 |             396 |
| max_latency  |            264 |            782 |           3201 |            3929 |
| min_latency  |             40 |             40 |             40 |              40 |
| p50_latency  |             42 |             61 |            103 |             312 |
| p90_latency  |             55 |            108 |            305 |             837 |
| p99_latency  |            158 |            309 |            413 |            1854 |