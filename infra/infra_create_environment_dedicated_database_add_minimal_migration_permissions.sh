#!/bin/bash

# Function to check if doctl is installed
check_doctl() {
  if ! command -v doctl &> /dev/null
  then
    echo "Error: doctl is not installed."
    exit 1
  fi
}

# Function to show usage instructions
usage() {
  echo "Usage: $0 {staging|development|production}"
  exit 1
}

# Function to update user permissions for migrations
update_user_permissions() {
  echo "Updating minimal migration permissions for database '$DB_NAME' in cluster '$CLUSTER_NAME'..."

  CLUSTER_ID=$(doctl databases list --format ID,Name --no-header | grep "$CLUSTER_NAME" | awk '{print $1}')

  if [ -z "$CLUSTER_ID" ]; then
    echo "Error: Cluster '$CLUSTER_NAME' not found."
    exit 1
  fi

  # SQL commands are passed directly to db-sql
  doctl databases db-sql "$CLUSTER_ID" "
    ALTER USER \"${DB_NAME}\" WITH CREATEDB;
    GRANT CREATE ON SCHEMA public TO \"${DB_NAME}\";
    GRANT USAGE ON SCHEMA public TO \"${DB_NAME}\";
  "

  if [ $? -eq 0 ]; then
    echo "Successfully updated permissions for database user '${DB_NAME}'."
  else
    echo "Error: Failed to update permissions."
    exit 1
  fi
}

# Main script execution
if [ $# -ne 1 ]; then
  usage
fi

ENV=$1
CLUSTER_NAME="${ENV}-cluster"
DB_NAME="$ENV"

check_doctl
update_user_permissions