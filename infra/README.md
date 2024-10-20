## DigitalOcean App Platform Commands

### Authentication
```bash
# List all available authentication contexts
doctl auth list

# Switch to a specific context
doctl auth switch --context your-context-name
```

### App management
```bash
# List all apps
doctl apps list

# Get app details
doctl apps get <app-id>

# Update app from spec file
doctl apps update <app-id> --spec spec.staging.yaml

# Create new app from spec
doctl apps create --spec spec.staging.yaml

# Delete an app
doctl apps delete <app-id>
```

### Spec File Management
```bash
# Get current spec (useful for seeing what DigitalOcean has configured)
doctl apps spec get <app-id> > current-spec.yaml

# Validate a spec file without deploying
doctl apps spec validate ./spec.staging.yaml
```