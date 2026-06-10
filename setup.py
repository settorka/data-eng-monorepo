import argparse
import os
import shutil

def create_project(tool, project, subfolders, clouds):
    # Define base paths
    tool_path = os.path.join(tool)
    project_path = os.path.join(tool_path, project)
    local_path = os.path.join(project_path, 'local')
    cloud_path = os.path.join(project_path, 'cloud')

    # Create the tool folder if it doesn't exist
    if not os.path.exists(tool_path):
        os.makedirs(tool_path)

    # Check if project exists and prompt for overwrite
    if os.path.exists(project_path):
        confirm = input(f"Project '{project}' already exists. Overwrite? (y/n): ")
        if confirm.lower() != 'y':
            print("Skipping project creation.")
            return
        else:
            shutil.rmtree(project_path)  # Delete existing project
            print(f"Deleted existing project {project_path}")
    else:
        print(f"Creating project {project_path}...")

    # Create project directory
    os.makedirs(local_path)
    os.makedirs(cloud_path)

    # Create subfolders inside local
    subfolder_list = subfolders.split(',')
    for subfolder in subfolder_list:
        subfolder_path = os.path.join(local_path, subfolder)
        os.makedirs(subfolder_path)

        # Create files inside the subfolder
        open(os.path.join(subfolder_path, 'Dockerfile'), 'w').close()
        open(os.path.join(subfolder_path, '.env'), 'w').close()
        open(os.path.join(subfolder_path, 'README.md'), 'w').close()

    # Create default local files (docker-compose.yml and README.md)
    open(os.path.join(local_path, 'docker-compose.yml'), 'w').close()
    open(os.path.join(local_path, 'README.md'), 'w').close()

    # Create clouds folders based on specified clouds
    cloud_list = clouds.split(',')
    for cloud in cloud_list:
        cloud_folder = os.path.join(cloud_path, cloud)
        os.makedirs(cloud_folder)

        # Create files inside each cloud folder
        open(os.path.join(cloud_folder, 'Dockerfile'), 'w').close()
        open(os.path.join(cloud_folder, '.env'), 'w').close()
        open(os.path.join(cloud_folder, 'README.md'), 'w').close()

    print(f"Project '{project}' created successfully.")

def delete_path(path, force=False):
    # Validate path exists
    if os.path.exists(path):
        confirm = input(f"Are you sure you want to delete {path}? (y/n): ") if not force else 'y'
        if confirm.lower() == 'y':
            if os.path.isdir(path):
                shutil.rmtree(path)  # Recursively delete directory
            else:
                os.remove(path)  # Delete file
            print(f"Deleted {path}")
        else:
            print(f"Skipping {path}")
    else:
        print(f"{path} does not exist.")

def rename_path(old_path, new_name):
    # Rename path
    if os.path.exists(old_path):
        parent_dir = os.path.dirname(old_path)
        new_path = os.path.join(parent_dir, new_name)
        if os.path.exists(new_path):
            print(f"Error: {new_path} already exists.")
        else:
            os.rename(old_path, new_path)
            print(f"Renamed {old_path} to {new_path}")
    else:
        print(f"{old_path} does not exist.")

def list_structure(path=None, full=False):
    # Print project structure recursively
    if not path:
        print("Listing all tools...")
    else:
        print(f"Listing structure for {path}...")
    
    for root, dirs, files in os.walk(path or "."):
        print(f"{root}/")
        for dir in dirs:
            print(f"  {dir}/")
        for file in files:
            print(f"  {file}")

def main():
    parser = argparse.ArgumentParser(description="Manage project scaffolding in a data engineering monorepo")

    subparsers = parser.add_subparsers(dest='command')  # <-- explicitly store command name

    # Create command
    create_parser = subparsers.add_parser('create', help='Create a new project')
    create_parser.add_argument('--tool', required=True, help="Tool name (e.g., hudi)")
    create_parser.add_argument('--project', required=True, help="Project name (e.g., write_upsert)")
    create_parser.add_argument('--subfolders', default="python", help="Comma-separated list of subfolders")
    create_parser.add_argument('--clouds', default="aws", help="Comma-separated list of cloud targets")
    create_parser.set_defaults(func=lambda args: create_project(args.tool, args.project, args.subfolders, args.clouds))

    # Delete command
    delete_parser = subparsers.add_parser('delete', help="Delete a project, tool, cloud, or subfolder")
    delete_parser.add_argument('path', help="Path to delete")
    delete_parser.add_argument('-y', action='store_true', help="Skip confirmation")
    delete_parser.set_defaults(func=lambda args: delete_path(args.path, args.y))

    # Rename command
    rename_parser = subparsers.add_parser('rename', help="Rename a tool, project, or subfolder")
    rename_parser.add_argument('old_path', help="Path to rename")
    rename_parser.add_argument('new_name', help="New name for the path")
    rename_parser.set_defaults(func=lambda args: rename_path(args.old_path, args.new_name))

    # List command
    list_parser = subparsers.add_parser('list', help="List the structure of the project")
    list_parser.add_argument('--tool', help="Tool name to list")
    list_parser.add_argument('--project', help="Project path to list")
    list_parser.add_argument('--full', action='store_true', help="List the full structure")
    list_parser.set_defaults(func=lambda args: list_structure(args.tool or args.project, args.full))

    # Parse args
    args = parser.parse_args()

    if hasattr(args, 'func'):
        args.func(args)
    else:
        parser.print_help()  # <-- Show help message if no subcommand is given


if __name__ == "__main__":
    main()
