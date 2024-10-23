#!/bin/bash

# Script to update a DigitalOcean app using doctl CLI.
# It requires the app ID and the environment to specify the correct spec file.

# Function to check if doctl is installed
check_doctl() {
  if ! command -v doctl &> /dev/null
  then
    echo "Error: doctl is not installed."
    echo "Please install doctl by following instructions at: https://docs.digitalocean.com/reference/doctl/how-to/install/"
    exit 1
  fi
}

# Function to show usage instructions
usage() {
  echo "Usage: $0 <APP_ID> <ENVIRONMENT>"
  exit 1
}

# Main script execution
if [ $# -ne 2 ]; then
  usage
fi

# Set the app ID and environment
APP_ID=$1
ENVIRONMENT=$2

# Check if doctl is installed
check_doctl

# Update the app using doctl with the corresponding spec file
doctl apps update "$APP_ID" --spec="infra/specs/spec.${ENVIRONMENT}.yaml"

# Check if the app update was successful
if [ $? -eq 0 ]; then
  echo "App with ID '$APP_ID' updated successfully using spec for '$ENVIRONMENT'."
else
  echo "Error: Failed to update the app with ID '$APP_ID'."
  exit 1
fi