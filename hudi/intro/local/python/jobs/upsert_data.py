"""
upsert_data.py

Demonstrates how to perform an upsert operation in Apache Hudi using PySpark.
The script writes an initial dataset to a Hudi table, then applies an upsert
consisting of both updated and new records. The table is read and displayed
before and after the upsert to verify Hudi's behavior.

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
    """
    Create initial dataset to be written to Hudi.
    """
    data = [
        {"id": "1", "name": "Alice", "ts": 1000},
        {"id": "2", "name": "Bob", "ts": 1001},
        {"id": "3", "name": "Charlie", "ts": 1002}
    ]
    return spark.createDataFrame(data)

def create_upsert_df(spark: SparkSession) -> DataFrame:
    """
    Create upsert dataset with updated and new records.
    - 'id=1' is updated (newer ts)
    - 'id=4' is a new record
    """
    data = [
        {"id": "1", "name": "AliceUpdated", "ts": 2000},
        {"id": "4", "name": "Dave", "ts": 1003}
    ]
    return spark.createDataFrame(data)

def write_to_hudi(spark: SparkSession, df: DataFrame, base_path: str, mode: str) -> None:
    """
    Write a DataFrame to Hudi with specified mode.

    Args:
        spark: SparkSession
        df: DataFrame to write
        base_path: S3A path for the Hudi table
        mode: Write mode ('append' or 'overwrite')
    """
    table_name = "users_table"
    hudi_options = get_hudi_options(
        table_name=table_name,
        record_key="id",
        precombine_key="ts"
    )
    logger.info(f"Writing to Hudi table at {base_path} with mode='{mode}'")
    df.write.format("hudi") \
        .options(**hudi_options) \
        .mode(mode) \
        .save(base_path)
    logger.info("Write complete")

def read_hudi_table(spark: SparkSession, base_path: str) -> DataFrame:
    """
    Read the current state of the Hudi table from the specified path.

    Args:
        spark: SparkSession
        base_path: Path to the Hudi table

    Returns:
        DataFrame containing the table contents
    """
    return spark.read.format("hudi").load(base_path)

if __name__ == "__main__":
    spark = SparkSession.builder \
        .appName("HudiUpsertDemo") \
        .config("spark.serializer", "org.apache.spark.serializer.KryoSerializer") \
        .config("spark.driver.extraJavaOptions", "-Dlog4j.configuration=file:/app/log4j.properties") \
        .config("spark.executor.extraJavaOptions", "-Dlog4j.configuration=file:/app/log4j.properties") \
        .getOrCreate()

    configure_s3a_for_minio(spark)

    base_path = "s3a://hudi-bucket/users_table"

    # Step 1: Write initial data
    initial_df = create_initial_df(spark)
    write_to_hudi(spark, initial_df, base_path, mode="overwrite")

    # Step 2: Read and display table before upsert
    logger.info("=== STATE BEFORE UPSERT ===")
    before_df = read_hudi_table(spark, base_path)
    before_df.show(truncate=False)

    # Step 3: Write upsert data
    upsert_df = create_upsert_df(spark)
    write_to_hudi(spark, upsert_df, base_path, mode="append")

    # Step 4: Read and display table after upsert
    logger.info("=== STATE AFTER UPSERT ===")
    after_df = read_hudi_table(spark, base_path)
    after_df.show(truncate=False)

    spark.stop()
