#!/usr/bin/env python3
"""
MinIO bucket management CLI tool.
Supports creating, listing, and renaming buckets using MinIO and dotenv.
"""

import os
import argparse
from minio import Minio
from minio.error import S3Error
from dotenv import load_dotenv

# Load env vars
load_dotenv()

# MinIO config
endpoint = os.getenv("MINIO_ENDPOINT")
access_key = os.getenv("MINIO_ACCESS_KEY")
secret_key = os.getenv("MINIO_SECRET_KEY")
secure = os.getenv("MINIO_SECURE", "false").lower() == "true"

# Initialize MinIO client
client = Minio(
    endpoint,
    access_key=access_key,
    secret_key=secret_key,
    secure=secure
)

def create_bucket(bucket_name):
    if client.bucket_exists(bucket_name):
        print(f"Bucket '{bucket_name}' already exists.")
    else:
        client.make_bucket(bucket_name)
        print(f"Bucket '{bucket_name}' created.")

def list_buckets():
    buckets = client.list_buckets()
    for b in buckets:
        print(f"{b.name} - created {b.creation_date}")

def rename_bucket(source, target):
    if not client.bucket_exists(source):
        print(f"Source bucket '{source}' does not exist.")
        return

    if client.bucket_exists(target):
        print(f"Target bucket '{target}' already exists.")
        return

    # Create target bucket
    client.make_bucket(target)

    # Copy objects
    objects = client.list_objects(source, recursive=True)
    for obj in objects:
        source_obj = f"/{source}/{obj.object_name}"
        client.copy_object(target, obj.object_name, source_obj)
        print(f"Copied: {obj.object_name}")

    # Delete all source objects
    delete_objs = [obj.object_name for obj in client.list_objects(source, recursive=True)]
    for name in delete_objs:
        client.remove_object(source, name)

    # Delete source bucket
    client.remove_bucket(source)
    print(f"Renamed '{source}' → '{target}'")

def main():
    parser = argparse.ArgumentParser(description="MinIO Bucket CLI")
    subparsers = parser.add_subparsers(dest="command")

    # Create
    create_parser = subparsers.add_parser("create", help="Create a new bucket")
    create_parser.add_argument("--bucket-name", required=True)

    # List
    subparsers.add_parser("list", help="List all buckets")

    # Rename
    rename_parser = subparsers.add_parser("rename", help="Rename a bucket")
    rename_parser.add_argument("--source", required=True)
    rename_parser.add_argument("--target", required=True)

    args = parser.parse_args()

    if args.command == "create":
        create_bucket(args.bucket_name)
    elif args.command == "list":
        list_buckets()
    elif args.command == "rename":
        rename_bucket(args.source, args.target)
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
