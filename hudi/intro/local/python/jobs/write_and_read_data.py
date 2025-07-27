from pyspark.sql import SparkSession
from config import get_hudi_options, configure_s3a_for_minio

def create_sample_df(spark):
    data = [
        {"id": "1", "name": "Alice", "ts": 1000},
        {"id": "2", "name": "Bob", "ts": 1001},
        {"id": "3", "name": "Charlie", "ts": 1002}
    ]
    return spark.createDataFrame(data)

def write_to_hudi(spark, df, base_path):
    table_name = "users_table"
    hudi_options = get_hudi_options(
        table_name=table_name,
        record_key="id",
        precombine_key="ts"
    )
    print(f"[INFO] Writing to Hudi table at {base_path}...")
    df.write.format("hudi") \
        .options(**hudi_options) \
        .mode("overwrite") \
        .save(base_path)
    print("[SUCCESS] Write complete.")

def read_from_hudi(spark, base_path):
    print(f"[INFO] Reading from Hudi table at {base_path}...")
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
