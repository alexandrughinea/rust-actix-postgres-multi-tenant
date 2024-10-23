#!/bin/bash

# Script to create a dedicated user for running SQLx migrations in a DigitalOcean PostgreSQL cluster.
# The user will be given minimal required permissions for running migrations.

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
  echo "Usage: $0 {environment} {migration_username} {migration_password}"
  echo "Defaults:"
  echo "  Environment: staging"
  echo "  Migration Username: migration_user"
  echo "  Migration Password: securepassword"
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

# Function to create the migration user
create_migration_user() {
  echo "Creating migration user '$MIGRATION_USER' for database '$DB_NAME' in cluster '$CLUSTER_NAME'..."

  # Create the user in the database cluster
  doctl databases user create "$CLUSTER_NAME" "$MIGRATION_USER"

  # Grant necessary permissions for SQLx migrations
  echo "Granting necessary privileges to migration user '$MIGRATION_USER' on database '$DB_NAME'..."

  # Run SQL commands to grant the required permissions
  doctl databases sql "$CLUSTER_NAME" \
    --username "$DEFAULT_ADMIN_USER" \
    --database "$DB_NAME" \
    --file - <<-EOSQL
      CREATE USER "$MIGRATION_USER" WITH PASSWORD '$MIGRATION_PASSWORD';
      GRANT CONNECT ON DATABASE "$DB_NAME" TO "$MIGRATION_USER";
      GRANT USAGE ON SCHEMA public TO "$MIGRATION_USER";
      GRANT CREATE, USAGE, ALTER, DROP ON ALL TABLES IN SCHEMA public TO "$MIGRATION_USER";
      GRANT CREATE, USAGE ON ALL SEQUENCES IN SCHEMA public TO "$MIGRATION_USER";
      ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT CREATE, ALTER, DROP ON TABLES TO "$MIGRATION_USER";
      GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO "$MIGRATION_USER";
EOSQL

  if [ $? -eq 0 ]; then
    echo "Migration user '$MIGRATION_USER' created successfully and privileges granted."
  else
    echo "Error: Failed to create migration user or grant privileges."
    exit 1
  fi
}

# Main script execution

# Set defaults if not provided
ENV=${1:-staging}                # Default environment: staging
MIGRATION_USER=${2:-migration_user}  # Default username: migration_user
MIGRATION_PASSWORD=${3:-securepassword} # Default password: securepassword

# Admin user for executing SQL commands
DEFAULT_ADMIN_USER="doadmin"  # Replace this with your actual admin username if necessary

# Check if doctl is installed
check_doctl

# Set environment-specific variables (cluster name and database name)
set_environment_config

# Create the migration user with the required permissions
create_migration_user

