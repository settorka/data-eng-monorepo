FROM bitnami/spark:3.4.1

USER root

# Install required system packages for PySpark and development
RUN install_packages \
    curl \
    wget \
    unzip \
    python3 \
    python3-pip \
    python3-setuptools \
    python3-dev \
    build-essential \
    git \
    nano \
    lsof \
    net-tools \
    iputils-ping

# Download Hudi and Hadoop AWS JARs
RUN mkdir -p /opt/spark/jars && \
    curl -o /opt/spark/jars/hudi-spark3.3-bundle.jar https://repo1.maven.org/maven2/org/apache/hudi/hudi-spark3.3-bundle_2.12/0.14.0/hudi-spark3.3-bundle_2.12-0.14.0.jar && \
    curl -o /opt/spark/jars/hadoop-aws-3.3.1.jar https://repo1.maven.org/maven2/org/apache/hadoop/hadoop-aws/3.3.1/hadoop-aws-3.3.1.jar && \
    curl -o /opt/spark/jars/aws-java-sdk-bundle-1.11.901.jar https://repo1.maven.org/maven2/com/amazonaws/aws-java-sdk-bundle/1.11.901/aws-java-sdk-bundle-1.11.901.jar

# Install Python packages
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
