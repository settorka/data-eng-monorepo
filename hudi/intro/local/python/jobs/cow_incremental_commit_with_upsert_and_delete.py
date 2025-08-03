"""
cow_incremental_commit_with_upsert_and_delete.py

Demonstrates Copy-On-Write (COW) table operations in Apache Hudi using PySpark:
1. Initial write (commit1)
2. Upsert (commit2)
3. Delete (commit3)
4. Incremental read of all changes since commit1

Assumes the Spark session is configured for S3A access to a MinIO-based object store,
and that job scripts are mounted at /app inside the container.
"""

import logging
from pyspark.sql import SparkSession, DataFrame
from config import get_hudi_options, configure_s3a_for_minio

# Configure logger with timestamp format
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s | %(levelname)s | %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S"
)
logger = logging.getLogger(__name__)

def create_initial_df(spark: SparkSession) -> DataFrame:
    """Create initial dataset to be written to Hudi."""
    data = [
        {"id": "1", "name": "Alice", "ts": 1000},
        {"id": "2", "name": "Bob", "ts": 1001},
        {"id": "3", "name": "Charlie", "ts": 1002}
    ]
    return spark.createDataFrame(data)

def create_upsert_df(spark: SparkSession) -> DataFrame:
    """Create upsert dataset with updated and new records."""
    data = [
        {"id": "1", "name": "AliceUpdated", "ts": 2000},
        {"id": "4", "name": "Dave", "ts": 1003}
    ]
    return spark.createDataFrame(data)

def create_delete_df(spark: SparkSession) -> DataFrame:
    """Create delete dataset with IDs of records to remove."""
    data = [
        {"id": "2"},
        {"id": "4"}
    ]
    return spark.createDataFrame(data)

def write_to_hudi(spark: SparkSession, df: DataFrame, base_path: str, mode: str, operation: str = "upsert") -> None:
    """
    Write a DataFrame to Hudi with specified mode and operation.

    Args:
        spark: SparkSession
        df: DataFrame to write
        base_path: S3A path for the Hudi table
        mode: Write mode ('append' or 'overwrite')
        operation: Hudi operation ('upsert' or 'delete')
    """
    table_name = "users_table"
    hudi_options = get_hudi_options(
        table_name=table_name,
        record_key="id",
        precombine_key="ts" if operation != "delete" else None
    )
    hudi_options["hoodie.datasource.write.operation"] = operation

    logger.info(f"Writing to Hudi table at {base_path} with mode='{mode}', operation='{operation}'")
    df.write.format("hudi") \
        .options(**hudi_options) \
        .mode(mode) \
        .save(base_path)
    logger.info("Write complete")

def read_hudi_table(spark: SparkSession, base_path: str) -> DataFrame:
    """Read the current state of the Hudi table."""
    return spark.read.format("hudi").load(base_path)

def read_incremental(spark: SparkSession, base_path: str, begin_time: str) -> DataFrame:
    """
    Perform an incremental read from a Hudi table starting at the given commit time.

    Args:
        spark: SparkSession
        base_path: Path to the Hudi table
        begin_time: Commit time (instant) to start reading from

    Returns:
        DataFrame of changes since begin_time
    """
    logger.info(f"Performing incremental read from commit time: {begin_time}")
    return spark.read.format("hudi") \
        .option("hoodie.datasource.query.type", "incremental") \
        .option("hoodie.datasource.read.begin.instanttime", begin_time) \
        .load(base_path)

if __name__ == "__main__":
    spark = SparkSession.builder \
        .appName("COWIncrementalUpsertDeleteDemo") \
        .config("spark.serializer", "org.apache.spark.serializer.KryoSerializer") \
        .config("spark.driver.extraJavaOptions", "-Dlog4j.configuration=file:/app/log4j.properties") \
        .config("spark.executor.extraJavaOptions", "-Dlog4j.configuration=file:/app/log4j.properties") \
        .getOrCreate()
    spark.sparkContext.setLogLevel('WARN')

    configure_s3a_for_minio(spark)

    base_path = "s3a://hudi-bucket/users_table"

    # Step 1: Initial write
    initial_df = create_initial_df(spark)
    write_to_hudi(spark, initial_df, base_path, mode="overwrite")
    commit1 = "00000000000001"  # Placeholder; will be actual instant time
    logger.info("=== STATE AFTER INITIAL WRITE ===")
    read_hudi_table(spark, base_path).show(truncate=False)

    # Step 2: Upsert
    upsert_df = create_upsert_df(spark)
    write_to_hudi(spark, upsert_df, base_path, mode="append")
    logger.info("=== STATE AFTER UPSERT ===")
    read_hudi_table(spark, base_path).show(truncate=False)

    # Step 3: Delete
    delete_df = create_delete_df(spark)
    write_to_hudi(spark, delete_df, base_path, mode="append", operation="delete")
    logger.info("=== STATE AFTER DELETE ===")
    read_hudi_table(spark, base_path).show(truncate=False)

    # Step 4: Incremental read (since commit1)
    logger.info("=== INCREMENTAL READ SINCE INITIAL COMMIT ===")
    incremental_df = read_incremental(spark, base_path, begin_time=commit1)
    incremental_df.show(truncate=False)

    spark.stop()
