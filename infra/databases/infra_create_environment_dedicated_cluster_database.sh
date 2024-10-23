#!/bin/bash

# Script to create a new database in a DigitalOcean cluster using doctl CLI.
# The cluster name is dynamically created by appending "-cluster" to the environment name.
# The database name is simply the environment name (staging, development, or production).

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
  echo "Usage: $0 {staging|development|production}"
  exit 1
}

# Function to set environment-specific variables
set_environment_config() {
  case "$ENV" in
    staging|development|production)
      CLUSTER_NAME="${ENV}-cluster"
      DB_NAME="$ENV"
      ;;
    *)
      echo "Error: Invalid environment specified."
      usage
      ;;
  esac
}

# Function to create a new database in the cluster
create_database() {
  echo "Creating database '$DB_NAME' in cluster '$CLUSTER_NAME'..."

  # Execute the doctl command to create the database
  doctl databases db create "$CLUSTER_NAME" "$DB_NAME"

  # Check if the database creation was successful
  if [ $? -eq 0 ]; then
    echo "Database '$DB_NAME' created successfully in the '$ENV' environment."
  else
    echo "Error: Failed to create the database in the '$ENV' environment."
    exit 1
  fi
}

# Main script execution
if [ $# -ne 1 ]; then
  usage
fi

# Set the environment
ENV=$1

# Check if doctl is installed
check_doctl

# Set environment-specific variables
set_environment_config

# Create the database
create_database
