#!/usr/bin/env bash

{
tree -a -I .git --gitignore
echo
for f in ./.env.example ./Cargo.toml ./src/**/*.rs "${@}" ; do
  echo "-------------------------------------------------------------------"
  echo "File: $f"
  echo "-------------------------------------------------------------------"
  cat -n "$f"
  echo "-------------------------------------------------------------------"
done
} | tee getfiles.txt | bat

