FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    g++ \
    cmake \
    git \
    iproute2 \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN mkdir build && cd build && cmake -DCMAKE_BUILD_TYPE=Release .. && make

ENTRYPOINT [ "/bin/bash", "entrypoint.sh" ]