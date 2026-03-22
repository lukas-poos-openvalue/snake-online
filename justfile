tools := "spacetime docker kubectl envsubst cog"
registry := "registry.luke-homelab.de"

default:
    @just --list

# Performs preliminary checks to guarantee that the other recipes can work properly
_check:
    @for tool in {{tools}}; do if ! command -v $tool >/dev/null 2>&1; then echo "$tool is not installed!"; exit 1; fi; done
    @if ! docker login --get-login {{registry}} >/dev/null 2>&1; then echo "Please log into container registry!"; exit 1; fi
    @if ! kubectl cluster-info >/dev/null 2>&1; then echo "Could not connect to k3s cluster!"; exit 1; fi

# Starts the development servers
dev: _check
    @spacetime dev --server local

# Determines the pre-release version, builds the images and starts the containers
dev-container: _check
    #!/usr/bin/env sh
    version=$(cog bump --dry-run --auto --pre "dev.*")
    just build-image $version local
    just run-container $version

# Determines the pre-release version, builds the image and deploys it as
dev-deploy: _check
    #!/usr/bin/env sh
    version=$(cog bump --dry-run --auto --pre "dev.*")
    just build-image $version
    just deploy $version

# Builds the frontend and the backend images
build-image version profile="": _check
    @cd backend; just build-image {{version}} {{profile}}
    @cd frontend; just build-image {{version}} {{profile}}

# Runs the containers locally (requires a previous build-image with the local profile)
run-container version: _check
    @cd backend; just run-container {{version}}
    @cd frontend; just run-container {{version}}

# Sets the version in the frontend & backend projects
set-version version: _check
    @cd backend; just set-version {{version}}
    @cd frontend; just set-version {{version}}
    
# Performs a deployment of the given version
deploy version: _check
    @cd backend; just push-image {{version}}
    @cd frontend; just push-image {{version}}

    version={{version}} envsubst < deployment.yaml | kubectl apply -f -
    kubectl get -f deployment.yaml -o name | grep '^deployment' | xargs -r kubectl rollout restart

# Performs a release
release: _check
    cog bump --auto

# Performs a preliminary release
pre-release: _check
    cog bump --auto --pre

generate:
    @cd backend; just generate

publish:
    @cd backend; just publish
