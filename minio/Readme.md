# Minio CLI
Local S3 store

## Example Usage
### Create a bucket
```bash
python minio_bucket.py create --bucket-name my-bucket
```
### List buckets
```bash
python minio_bucket.py list
```

### Rename a bucket
```bash
python minio_bucket.py rename --source old-name --target new-name
```

## Setup
### pip
1. Create the venv
```bash
python3 -m venv minio-venv
```

2. Activate it
```bash
source minio-venv/bin/activate
```

3. 
- Install requirements.txt
```bash
pip install -r requirements.txt
```

- Deactivate venv
```bash
deactivate
```
### uv
0. Install pipx ([sudo apt install](https://pipx.pypa.io/stable/installation/)) and pipx install uv
1. Create the venv
```bash
uv venv minio-venv
```

1. Activate it
```bash
source minio-venv/bin/activate
```

1. 
- Install requirements.txt
```bash
uv pip install -r requirements.txt
```

- Deactivate venv
```bash
deactivate
```