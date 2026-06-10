# data-eng-monorepo
Monorepo for data engineering tools and projects

## Getting Started

### Installation

```bash
git clone <repository_url>
cd data-eng-monorepo
pip install -r requirements.txt
```
Ensure you have Python 3.x installed.
### CLI Commands

#### Create a New Project
- Scaffold a new tool and project:
```bash
python3 setup.py create
  --tool hudi
  --project write_upsert 
  --subfolders python,java
  --clouds aws,gcp
```
- --tool Name of the tool (e.g. hudi, flink)
 
- --project Name of the project (e.g. write_upsert)

- --subfolders Comma-separated local subfolders under local/ (default: python)

- --clouds Comma-separated cloud targets under cloud/ (default: aws)

#### List Project Structure

Inspect your tools, projects, subfolders and clouds:
```bash
python3 setup.py list [--tool <tool_name>] [--project <tool>/<project>] [--full]

    No flags → lists all tools

    --tool hudi → lists all projects under hudi/

    --project hudi/write_upsert → lists local/ and cloud/ under that project

    --full → shows the full nested tree
```
Example output:

```bash
hudi/
└── write_upsert/
    ├── local/
    │   ├── python/
    │   └── java/
    └── cloud/
        ├── aws/
        └── gcp/
```

#### Rename a Project or Tool

Rename any tool, project, cloud, or local subfolder:
```bash
python3 setup.py rename <existing_path> <new_name>
```

Examples:

##### Rename the tool folder hudi → hoodie\
python3 setup.py rename hudi hoodie

##### Rename the project write_upsert → write_v2\
python3 setup.py rename hudi/write_upsert write_v2

##### Rename the cloud target gcp → azure\
python3 setup.py rename hudi/write_upsert/cloud/gcp azure

##### Rename a local subfolder java → scala\
python3 setup.py rename hudi/write_upsert/local/java scala

#### Delete a Project or Tool

Delete a tool, project, cloud target, or local subfolder:

```bash
python3 setup.py delete <path> [-y]

    <path> can be:

        hudi (tool)

        hudi/write_upsert (project)

        hudi/write_upsert/cloud/aws (cloud target)

        hudi/write_upsert/local/python (local subfolder)

    -y skip confirmation
```

Example:
```bash
python3 setup.py delete hudi-2 -y
```
### Example Workflow

1. Create
```bash
python3 setup.py create --tool hudi --project write_upsert --subfolders python,java --clouds aws,gcp
```

2. List
```bash
python3 setup.py list --tool hudi
```

3. Rename
```bash
python3 setup.py rename hudi/write_upsert hudi-2
```

4. Delete
```bash
python3 setup.py delete hudi-2 -y
```
