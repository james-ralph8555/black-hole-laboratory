# Default Docker image name and tag
IMAGE_NAME ?= black-hole-laboratory
IMAGE_TAG ?= latest

# Phony targets don't represent files
.PHONY: all build clean

# Default target
all: build

# Build the web application inside a Docker container
build:
	@echo "--- Building Docker image: $(IMAGE_NAME):$(IMAGE_TAG) ---"
	@docker build -t $(IMAGE_NAME):$(IMAGE_TAG) .
	@echo "\n--- Building web application ---"
	@docker run --rm -v "$(shell pwd)":/app -w /app/www $(IMAGE_NAME):$(IMAGE_TAG) \
		sh -c "nix develop -i --command 'npm install && npm run build'"

# Clean up build artifacts
clean:
	@echo "--- Cleaning build artifacts ---"
	@rm -rf www/dist
	@rm -rf www/node_modules
