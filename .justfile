set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

alias p := push
alias c := check
alias b := build
alias uv := update-version

default:
  just --list

tag TAG:
  git tag {{ TAG }}
  git push origin {{ TAG }}

del-tag TAG:
  git tag -d {{ TAG }}
  git push origin :refs/tags/{{ TAG }}

del-branch BRANCH:
  git branch -d {{ BRANCH }}

# git push with commit
push MESSAGE:
  git add .
  git commit -m "{{ MESSAGE }}"
  git push

reset:
  git reset --soft HEAD^

# prek check all files
check:
  prek run -a

dev:
  pnpm tauri dev

build:
  pnpm tauri build

fmt:
  cd src-tauri && cargo fmt

update-deps:
  pnpm upgrade
  pnpm update
  pnpm i
  cargo upgrade --manifest-path src-tauri/Cargo.toml
  cargo update --manifest-path src-tauri/Cargo.toml
  
[windows]
update-version VERSION:
  python scripts/update.py -v {{ VERSION }}

[linux]
update-version VERSION:
  uv run scripts/update.py -v {{ VERSION }}
