# Default Docker image name and tag
IMAGE_NAME ?= black-hole-laboratory
IMAGE_TAG ?= latest

# Phony targets don't represent files
.PHONY: build clean

# Build the web application. This is the target that AWS Amplify will run
# inside the custom build container.
build:
	@echo "--- Building web application ---"
	@nix develop --command sh -c 'cd www && npm install && npm run build'

# Clean up build artifacts
clean:
	@echo "--- Cleaning build artifacts ---"
	@rm -rf www/dist
	@rm -rf www/node_modules

