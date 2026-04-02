default:
    @just --list

# Starts the full application in development mode
dev:
    @spacetime dev --server local --delete-data

# Build the application parts
build-prod:
    @cd backend; just build-prod
    @cd frontend; just build-prod

# Build the application parts in development mode
build-dev:
    @cd backend; just build-dev
    @cd frontend; just build-dev

# Perform code generation
generate:
    @cd backend; just generate

# Publish the application locally
publish-local:
    @cd backend; just publish-local

# Deploys the application
deploy: build-prod
    @cd backend; just deploy
    @cd frontend; just deploy
