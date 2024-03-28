# Websocket benchmark

- Reduce network
  ```
  tc qdisc add dev eth0 root netem delay 20ms
  ```
- Set some limits (Linux only)
  ```
  sysctl -w fs.file-max=1000000
  sysctl -w fs.nr_open=1000000
  sysctl -w net.ipv4.tcp_mem="100000000 100000000 100000000"
  ```
