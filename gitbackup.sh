#!/bin/bash

# Get the current timestamp
timestamp=$(date +"%Y-%m-%d %H:%M:%S")

# Add all changes
git add .

# Commit with timestamp
git commit -m "Auto-commit: $timestamp"

# Push to the current branch
git push
