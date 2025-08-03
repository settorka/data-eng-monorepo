"""
mor_schema_evolution.py

Demonstrates a simplified Merge-On-Read (MOR) workflow in Apache Hudi using PySpark.

Flow:
1. Create an MOR table with an initial schema (id, name, ts) using bulk_insert.
2. Perform schema evolution by adding a new column (email) and upserting updated/new records.
3. Use inline compaction so that new Avro delta log entries are merged automatically into Parquet base files.
4. Read the table in:
   - Snapshot view (base files + any unmerged delta logs)
   - Read-Optimized view (compacted Parquet base files only)

This example uses:
- MERGE_ON_READ table type
- Inline compaction for simplicity
- Direct marker type to avoid timeline-server dependencies in local/minimal setups

Assumes:
- Spark session is configured for S3A access to a MinIO-based object store.
- Job scripts are mounted at /app inside the container.
"""

import logging
import sys
from pyspark.sql import SparkSession, DataFrame
from config import configure_s3a_for_minio  # existing MinIO/S3A helper

# Logger setup
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s | %(levelname)s | %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S"
)
logger = logging.getLogger(__name__)

BASE_PATH = "s3a://hudi-bucket/users_mor_evolution_table"
TABLE_NAME = "users_mor_evolution_table"

def create_initial_df(spark: SparkSession) -> DataFrame:
    """Initial dataset with schema: id, name, ts."""
    data = [
        {"id": "1", "name": "Alice", "ts": 1000},
        {"id": "2", "name": "Bob", "ts": 1001}
    ]
    return spark.createDataFrame(data)

def create_evolved_df(spark: SparkSession) -> DataFrame:
    """Evolved dataset adding new column 'email'."""
    data = [
        {"id": "3", "name": "Charlie", "ts": 1002, "email": "charlie@example.com"},
        {"id": "1", "name": "AliceUpdated", "ts": 2000, "email": "alice.updated@example.com"}
    ]
    return spark.createDataFrame(data)

def write_to_hudi_simple(
    spark: SparkSession,
    df: DataFrame,
    base_path: str,
    mode: str,
    operation: str = "upsert"
) -> None:
    """
    Minimal MOR write with inline compaction and direct markers.
    """
    hudi_options = {
        "hoodie.table.name": TABLE_NAME,
        "hoodie.datasource.write.table.name": TABLE_NAME,
        "hoodie.datasource.write.table.type": "MERGE_ON_READ",
        "hoodie.datasource.write.operation": operation,
        "hoodie.datasource.write.recordkey.field": "id",
        "hoodie.datasource.write.precombine.field": "ts",
        "hoodie.compact.inline": "true",
        "hoodie.compact.inline.max.delta.commits": "1",
        "hoodie.write.markers.type": "DIRECT",
        "hoodie.upsert.shuffle.parallelism": "2",
        "hoodie.insert.shuffle.parallelism": "2",
    }

    logger.info(f"[{operation.upper()}] Writing to Hudi MERGE_ON_READ at {base_path} with mode={mode}")
    try:
        df.write.format("hudi") \
            .options(**hudi_options) \
            .mode(mode) \
            .save(base_path)
        logger.info("Write complete")
    except Exception as e:
        logger.error(f"Write failed: {e}")
        raise

def read_snapshot(spark: SparkSession, base_path: str) -> DataFrame:
    """Snapshot view (base + logs)."""
    return spark.read.format("hudi").load(base_path)

def read_read_optimized(spark: SparkSession, base_path: str) -> DataFrame:
    """Read-optimized view (post-compaction base files)."""
    return spark.read.format("hudi") \
        .option("hoodie.datasource.query.type", "read_optimized") \
        .load(base_path)

def main():
    spark = SparkSession.builder \
        .appName("MORSchemaEvolutionKISS") \
        .config("spark.serializer", "org.apache.spark.serializer.KryoSerializer") \
        .config("spark.driver.extraJavaOptions", "-Dlog4j.configuration=file:/app/log4j.properties") \
        .config("spark.executor.extraJavaOptions", "-Dlog4j.configuration=file:/app/log4j.properties") \
        .getOrCreate()
    spark.sparkContext.setLogLevel('WARN')

    configure_s3a_for_minio(spark)

    try:
        # Step 1: Initial MOR write (bulk_insert to establish table)
        initial_df = create_initial_df(spark)
        write_to_hudi_simple(spark, initial_df, BASE_PATH, mode="overwrite", operation="bulk_insert")
        logger.info("=== STATE AFTER INITIAL WRITE (SNAPSHOT) ===")
        read_snapshot(spark, BASE_PATH).show(truncate=False)

        # Step 2: Schema evolution - upsert with new 'email' column
        evolved_df = create_evolved_df(spark)
        write_to_hudi_simple(spark, evolved_df, BASE_PATH, mode="append", operation="upsert")
        logger.info("=== STATE AFTER SCHEMA EVOLUTION (SNAPSHOT) ===")
        read_snapshot(spark, BASE_PATH).show(truncate=False)

        # Step 3: Read-optimized view: inline compaction should have happened already
        logger.info("=== STATE AFTER INLINE COMPACTION (READ-OPTIMIZED) ===")
        read_read_optimized(spark, BASE_PATH).show(truncate=False)

    except Exception as e:
        logger.error(f"Job failed: {e}")
        sys.exit(1)
    finally:
        spark.stop()

if __name__ == "__main__":
    main()
