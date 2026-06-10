def get_hudi_options(table_name, record_key, precombine_key, table_type="COPY_ON_WRITE"):
    return {
        "hoodie.table.name": table_name,
        "hoodie.datasource.write.recordkey.field": record_key,
        "hoodie.datasource.write.precombine.field": precombine_key,
        "hoodie.datasource.write.table.name": table_name,
        "hoodie.datasource.write.operation": "upsert",
        "hoodie.datasource.write.table.type": table_type,
        "hoodie.datasource.write.hive.style.partitioning": "false",
    }

def configure_s3a_for_minio(spark_session):
    hadoop_conf = spark_session._jsc.hadoopConfiguration()
    hadoop_conf.set("fs.s3a.endpoint", "http://minio:9000")
    hadoop_conf.set("fs.s3a.access.key", "minioadmin")
    hadoop_conf.set("fs.s3a.secret.key", "minioadmin")
    hadoop_conf.set("fs.s3a.path.style.access", "true")
    hadoop_conf.set("fs.s3a.impl", "org.apache.hadoop.fs.s3a.S3AFileSystem")
