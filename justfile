default:
    @just --list

# Starts the full application in development mode
dev:
    @spacetime dev --server local --delete-data

# Build the application
build:
    @cd backend; just build
    @cd frontend; just build

# Perform code generation
generate:
    @cd backend; just generate

# Publish the application locally
publish:
    @cd backend; just publish

# Deploys the application
deploy: build
    @cd backend; just deploy
    @cd frontend; just deploy
