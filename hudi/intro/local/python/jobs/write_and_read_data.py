"""
write_and_read_data.py

Writes a sample dataset to an Apache Hudi table using PySpark, then reads it back
to verify the write. The write operation uses overwrite mode and assumes a Copy-On-Write
(COW) table. Spark is configured to access MinIO using the S3A protocol.

Assumes the environment is correctly set up with S3A, Hudi dependencies, and logging.
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

def create_sample_df(spark: SparkSession) -> DataFrame:
    """
    Create a static sample DataFrame containing test records.

    Args:
        spark: SparkSession instance

    Returns:
        A DataFrame with test user data
    """
    data = [
        {"id": "1", "name": "Alice", "ts": 1000},
        {"id": "2", "name": "Bob", "ts": 1001},
        {"id": "3", "name": "Charlie", "ts": 1002}
    ]
    return spark.createDataFrame(data)

def write_to_hudi(spark: SparkSession, df: DataFrame, base_path: str) -> None:
    """
    Write the given DataFrame to a Hudi table using overwrite mode.

    Args:
        spark: SparkSession instance
        df: DataFrame to write
        base_path: S3A path where the Hudi table will be stored
    """
    table_name = "users_table"
    hudi_options = get_hudi_options(
        table_name=table_name,
        record_key="id",
        precombine_key="ts"
    )
    logger.info(f"Writing to Hudi table at {base_path} with overwrite mode")
    df.write.format("hudi") \
        .options(**hudi_options) \
        .mode("overwrite") \
        .save(base_path)
    logger.info("Write complete")

def read_from_hudi(spark: SparkSession, base_path: str) -> None:
    """
    Read a Hudi table from the given path and display its contents.

    Args:
        spark: SparkSession instance
        base_path: S3A path to the Hudi table
    """
    logger.info(f"Reading from Hudi table at {base_path}")
    df = spark.read.format("hudi").load(base_path)
    df.show(truncate=False)

if __name__ == "__main__":
    spark = SparkSession.builder \
        .appName("HudiInteractions") \
        .config("spark.serializer", "org.apache.spark.serializer.KryoSerializer") \
        .config("spark.driver.extraJavaOptions", "-Dlog4j.configuration=file:/app/log4j.properties") \
        .config("spark.executor.extraJavaOptions", "-Dlog4j.configuration=file:/app/log4j.properties") \
        .getOrCreate()

    configure_s3a_for_minio(spark)

    base_path = "s3a://hudi-bucket/users_table"
    df = create_sample_df(spark)
    write_to_hudi(spark, df, base_path)
    read_from_hudi(spark, base_path)

    spark.stop()
