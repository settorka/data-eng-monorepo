import subprocess
import argparse
import os
import sys

DOCKER_IMAGE = "hudi-spark"
SPARK_MASTER_URL = "spark://spark-master:7077"
NETWORK_NAME = "spark_hudi-net"
SCRIPT_DIR = os.path.join(os.getcwd(), "jobs")
CONTAINER_SCRIPT_DIR = "/app"

def run_spark_job(script_name):
    script_path = os.path.join(SCRIPT_DIR, script_name)
    if not os.path.exists(script_path):
        print(f"[ERROR] Script {script_name} not found in 'jobs/' folder.")
        sys.exit(1)

    docker_cmd = [
        "docker", "run", "--rm",
        "--network", NETWORK_NAME,
        "-v", f"{os.path.abspath(SCRIPT_DIR)}:{CONTAINER_SCRIPT_DIR}",
        DOCKER_IMAGE,
        "spark-submit",
        "--master", SPARK_MASTER_URL,
        "--packages", "org.apache.hudi:hudi-spark3.4-bundle_2.12:0.14.0",
        f"{CONTAINER_SCRIPT_DIR}/{script_name}"
    ]

    print(f"[INFO] Running {script_name}...")
    process = subprocess.Popen(docker_cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)

    for line in process.stdout:
        print(line, end="")

    process.wait()
    if process.returncode == 0:
        print(f"[SUCCESS] Job {script_name} completed.")
    else:
        print(f"[FAILURE] Job {script_name} exited with code {process.returncode}.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Submit Spark jobs to the Docker cluster")
    parser.add_argument("--filename", required=True, help="Name of the PySpark script in jobs/")

    args = parser.parse_args()
    run_spark_job(args.filename)
