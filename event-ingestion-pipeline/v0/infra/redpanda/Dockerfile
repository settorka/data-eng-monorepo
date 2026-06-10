FROM redpandadata/redpanda:v24.3.2

EXPOSE 9092 9644

ENTRYPOINT ["/usr/bin/redpanda", "start",
    "--smp", "1",
    "--memory", "1G",
    "--reserve-memory", "0M",
    "--overprovisioned",
    "--node-id", "0",
    "--check=false",
    "--kafka-addr", "PLAINTEXT://0.0.0.0:9092",
    "--advertise-kafka-addr", "PLAINTEXT://redpanda:9092",
    "--rpc-addr", "0.0.0.0:33145",
    "--advertise-rpc-addr", "redpanda:33145"
]
