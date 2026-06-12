import os
import json
import argparse
import tomllib

import toml


def main():
    parser = argparse.ArgumentParser(description="updater")
    parser.add_argument(
        "-v",
        type=str,
        help="version information",
    )
    args = parser.parse_args()
    version = args.v
    # update Cargo.toml
    print("updating Cargo.toml")
    with open("src-tauri/Cargo.toml", "rb") as f:
        cargo_data = tomllib.load(f)
    cargo_data["package"]["version"] = version
    with open("src-tauri/Cargo.toml", "w") as f:
        toml.dump(cargo_data, f)
    # update tauri.conf.json
    print("updating tauri.conf.json")
    with open("src-tauri/tauri.conf.json", "r") as f:
        tauri_data = json.load(f)
    tauri_data["version"] = version
    with open("src-tauri/tauri.conf.json", "w") as f:
        json.dump(tauri_data, f, indent=2)
    # update package.json
    print("updating package.json")
    with open("package.json", "r") as f:
        package_data = json.load(f)
    package_data["version"] = version
    with open("package.json", "w") as f:
        json.dump(package_data, f, indent=2)
    os.system("just fmt")


if __name__ == "__main__":
    main()
