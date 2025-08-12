# Default Docker image name and tag
IMAGE_NAME ?= black-hole-laboratory
IMAGE_TAG ?= latest

# Phony targets don't represent files
.PHONY: all build clean build-image build-local

# Default target for local development. Builds the image and then the app.
all: build-local

# Build the web application. This is the target that AWS Amplify will run
# inside the custom build container.
build:
	@echo "--- Building web application ---"
	@nix develop --command sh -c 'cd www && npm install && npm run build'

# Build the Docker image.
build-image:
	@echo "--- Building Docker image: $(IMAGE_NAME):$(IMAGE_TAG) ---"
	@docker build -t $(IMAGE_NAME):$(IMAGE_TAG) .

# Run the application build process locally inside the Docker container.
# This simulates the AWS Amplify build environment.
build-local: build-image
	@echo "--- Building web application inside Docker ---"
	@docker run --rm -v "$(shell pwd)":/app $(IMAGE_NAME):$(IMAGE_TAG) make build

# Clean up build artifacts
clean:
	@echo "--- Cleaning build artifacts ---"
	@rm -rf www/dist
	@rm -rf www/node_modules
