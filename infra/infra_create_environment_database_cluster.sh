#!/bin/bash

# Script to create a PostgreSQL database cluster in DigitalOcean using doctl CLI.
# It supports different environments: staging, development, and production.
# Cluster configurations are environment-specific, such as size, region, and node count.

# Function to check if doctl is installed
check_doctl() {
  if ! command -v doctl &> /dev/null; then
    echo "Error: doctl is not installed."
    echo "Please install doctl by following instructions at: https://docs.digitalocean.com/reference/doctl/how-to/install/"
    exit 1
  fi
}

# Function to show usage instructions
usage() {
  echo "Usage: $0 <environment> <region> <size> <num_nodes> <engine> <version>"
  exit 1
}

# Function to check if the database cluster already exists
check_cluster_exists() {
  if doctl databases list | grep -q "$CLUSTER_NAME"; then
    echo "Cluster '$CLUSTER_NAME' already exists. Skipping creation."
    exit 0
  fi
}

# Function to create the database cluster
create_database_cluster() {
  echo "Creating $ENGINE database cluster '$CLUSTER_NAME' in the '$ENV' environment..."

  # Execute the doctl command to create the database cluster
  doctl databases create "$CLUSTER_NAME" \
    --engine "$ENGINE" \
    --num-nodes "$NUM_NODES" \
    --size "$SIZE" \
    --region "$REGION" \
    --version "$VERSION"

  # Check if the cluster creation was successful
  if [ $? -eq 0 ]; then
    echo "$ENGINE database cluster '$CLUSTER_NAME' created successfully in the '$ENV' environment (region: '$REGION')"
  else
    echo "Error: Failed to create the database cluster in the '$ENV' environment."
    exit 1
  fi
}

# Main script execution
if [ $# -ne 6 ]; then
  usage
fi

# Set the environment
ENV=$1
REGION=$2
SIZE=$3
NUM_NODES=$4
ENGINE=$5
VERSION=$6

# Check if doctl is installed
check_doctl

# Set the cluster name based on the environment
CLUSTER_NAME="${ENV}-cluster"

# Check if the cluster already exists
check_cluster_exists

# Create the database cluster
create_database_cluster