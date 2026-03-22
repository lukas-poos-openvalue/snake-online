default:
    just --list

# Start the backend in development mode using the "spacetime dev" command
dev-backend:
    spacetime dev --server local --client-lang typescript --module-path backend --module-bindings-path frontend/src/modules_bindings

# Start the frontend in development mode using the "ng serve" command
dev-frontend:
    cd frontend && ng serve --open

# Start the application in development mode
dev: dev-backend dev-frontend

# Generate the module bindings
generate:
    spacetime generate --lang typescript --module-path backend --out-dir frontend/src/modules_bindings

# Build the module
build:
    spacetime build --module-path backend

# Publish the module to local database
publish:
    spacetime publish --server local --module-path backend --delete-data spacetime-demo
