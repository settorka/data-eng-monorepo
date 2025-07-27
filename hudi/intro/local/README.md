# Apache Hudi interaction with Spark
## Project Structure
```bash
spark/                       # Spark infra and runner
├── docker-compose.yml       # Launches Spark master/worker + MinIO
├── Dockerfile               # Custom Spark image with Hudi + AWS jars
├── spark_cluster_interaction.py  # CLI to submit PySpark jobs to Spark
├── jobs/                    # PySpark job scripts
│   ├── config.py            # S3/Hudi write config
│   └── hudi_interactions.py # Example Hudi read/write
└── minio/data/              # Backing store for S3a (MinIO)
```

##  Setup & Run Instructions
1. Start the Cluster

From the spark/ directory:
```bash
docker compose up -d
```
This launches:
- Spark Master
- Spark Worker (connects to master)
- MinIO (S3-compatible blob store)
- Network: spark_hudi-net (auto-managed)

2. Submit a Spark Job

Use the CLI tool to submit a script in hudi/intro/jobs/:
```bash
python spark_cluster_interaction.py --filename hudi_interactions.py
```

Under the hood, it does:
```bash
docker run --rm \
  --network=spark_hudi-net \
  -v "$PWD/jobs/hudi_interactions.py":/app/hudi_interactions.py \
  hudi-spark \
  spark-submit \
    --master spark://spark-master:7077 \
    --packages org.apache.hudi:hudi-spark3.3-bundle_2.12:0.14.0 \
    /app/hudi_interactions.py
```

## Data Storage
Hudi writes to: s3a://hudi-bucket/<table_name>
Backed by: ./minio/data/ on host
MinIO UI: http://localhost:9001
(Login: minioadmin / minioadmin)

## Logs & Output

Logs stream to your terminal via spark_cluster_interaction.py

Output includes:

- PySpark show() table prints

- [SUCCESS] or [FAILURE] on exit

