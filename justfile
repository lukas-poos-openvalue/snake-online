default:
    @just --list

# Starts the full application in development mode
dev:
    @spacetime dev --server local --delete-data

# Build the application
build:
    @cd backend; just build
    @cd frontend; just build

# Build the application parts in debug mode
build-debug:
    @cd backend; just build-debug
    @cd frontend; just build-debug

# Perform code generation
generate:
    @cd backend; just generate

# Publish the application locally
publish-local:
    @cd backend; just publish-local

# Deploys the application
deploy: build
    @cd backend; just deploy
    @cd frontend; just deploy
