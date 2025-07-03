#!/bin/bash

# File Preview Service - Build and Test Script
# This script builds, tests, and deploys the Rust file preview service

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running on supported OS
check_os() {
    print_status "Checking operating system..."
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        print_success "Linux detected"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        print_success "macOS detected"
    else
        print_error "Unsupported operating system: $OSTYPE"
        exit 1
    fi
}

# Install system dependencies
install_dependencies() {
    print_status "Installing system dependencies..."
    
    if command -v apt-get &> /dev/null; then
        # Ubuntu/Debian
        sudo apt-get update
        sudo apt-get install -y \
            imagemagick \
            ffmpeg \
            unoconv \
            curl \
            libreoffice \
            pkg-config \
            libssl-dev \
            build-essential
            
        # Fix ImageMagick policy for PDF processing
        if [ -f /etc/ImageMagick-6/policy.xml ]; then
            sudo sed -i 's/<policy domain="coder" rights="none" pattern="PDF" \/>/<policy domain="coder" rights="read|write" pattern="PDF" \/>/g' /etc/ImageMagick-6/policy.xml
            print_success "ImageMagick PDF policy updated"
        fi
        
    elif command -v brew &> /dev/null; then
        # macOS with Homebrew
        brew install imagemagick ffmpeg curl
        print_warning "LibreOffice and unoconv need to be installed manually on macOS"
        print_warning "You can install LibreOffice from: https://www.libreoffice.org/download/"
        
    else
        print_error "Package manager not found. Please install dependencies manually."
        exit 1
    fi
    
    print_success "System dependencies installed"
}

# Install Rust if not present
install_rust() {
    if ! command -v cargo &> /dev/null; then
        print_status "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
        print_success "Rust installed"
    else
        print_success "Rust already installed"
    fi
}

# Build the project
build_project() {
    print_status "Building the project..."
    
    # Build in release mode for better performance
    cargo build --release
    
    print_success "Project built successfully"
}

# Run tests
run_tests() {
    print_status "Running tests..."
    
    # Run unit tests
    cargo test
    
    print_success "All tests passed"
}

# Start the service for testing
start_service() {
    print_status "Starting the service for testing..."
    
    # Set environment variables
    export RUST_LOG="filepreview_rust=debug,tower_http=debug"
    
    # Start the service in background
    cargo run --release &
    SERVICE_PID=$!
    
    # Wait for service to start
    sleep 5
    
    # Check if service is running
    if kill -0 $SERVICE_PID 2>/dev/null; then
        print_success "Service started successfully (PID: $SERVICE_PID)"
        echo $SERVICE_PID > .service_pid
    else
        print_error "Failed to start service"
        exit 1
    fi
}

# Test the service endpoints
test_endpoints() {
    print_status "Testing service endpoints..."
    
    # Test health endpoint
    if curl -f -s http://localhost:3000/health > /dev/null; then
        print_success "Health endpoint OK"
    else
        print_error "Health endpoint failed"
        return 1
    fi
    
    # Test capabilities endpoint
    if curl -f -s http://localhost:3000/ > /dev/null; then
        print_success "Root endpoint OK"
    else
        print_error "Root endpoint failed"
        return 1
    fi
    
    print_success "All endpoint tests passed"
}

# Test with sample file
test_sample_preview() {
    print_status "Testing preview generation..."
    
    # Create a simple test image
    if command -v convert &> /dev/null; then
        convert -size 100x100 xc:red test_image.png
        
        # Test preview generation with base64
        BASE64_IMAGE=$(base64 -w 0 test_image.png)
        
        RESPONSE=$(curl -s -X POST http://localhost:3000/preview \
            -H "Content-Type: application/json" \
            -d "{
                \"input\": \"data:image/png;base64,$BASE64_IMAGE\",
                \"output_format\": \"jpg\",
                \"options\": {
                    \"width\": 50,
                    \"height\": 50,
                    \"quality\": 85
                }
            }")
        
        if echo "$RESPONSE" | grep -q "success.*true"; then
            print_success "Preview generation test passed"
        else
            print_error "Preview generation test failed"
            echo "Response: $RESPONSE"
            return 1
        fi
        
        # Clean up test file
        rm -f test_image.png
    else
        print_warning "ImageMagick not available, skipping preview test"
    fi
}

# Stop the service
stop_service() {
    if [ -f .service_pid ]; then
        SERVICE_PID=$(cat .service_pid)
        if kill -0 $SERVICE_PID 2>/dev/null; then
            print_status "Stopping service (PID: $SERVICE_PID)..."
            kill $SERVICE_PID
            sleep 2
            
            # Force kill if still running
            if kill -0 $SERVICE_PID 2>/dev/null; then
                kill -9 $SERVICE_PID
            fi
            
            print_success "Service stopped"
        fi
        rm -f .service_pid
    fi
}

# Build Docker image
build_docker() {
    print_status "Building Docker image..."
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker not found. Please install Docker first."
        return 1
    fi
    
    docker build -t filepreview-rust:latest .
    print_success "Docker image built successfully"
}

# Test Docker container
test_docker() {
    print_status "Testing Docker container..."
    
    # Run container in background
    docker run -d --name filepreview-rust-test -p 3001:3000 filepreview-rust:latest
    
    # Wait for container to start
    sleep 10
    
    # Test health endpoint
    if curl -f -s http://localhost:3001/health > /dev/null; then
        print_success "Docker container test passed"
    else
        print_error "Docker container test failed"
        docker logs filepreview-rust-test
        return 1
    fi
    
    # Clean up
    docker stop filepreview-rust-test
    docker rm filepreview-rust-test
}

# Deploy with docker-compose
deploy_compose() {
    print_status "Deploying with docker-compose..."
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "docker-compose not found. Please install docker-compose first."
        return 1
    fi
    
    docker-compose up -d --build
    
    # Wait for service to be ready
    sleep 15
    
    # Test the deployed service
    if curl -f -s http://localhost:3000/health > /dev/null; then
        print_success "Docker Compose deployment successful"
        print_status "Service is running at http://localhost:3000"
    else
        print_error "Docker Compose deployment failed"
        docker-compose logs
        return 1
    fi
}

# Deploy to Kubernetes
deploy_k8s() {
    print_status "Deploying to Kubernetes..."
    
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl not found. Please install kubectl first."
        return 1
    fi
    
    # Apply Kubernetes manifests
    kubectl apply -f k8s-deployment.yaml
    
    # Wait for deployment to be ready
    kubectl rollout status deployment/filepreview-rust --timeout=300s
    
    print_success "Kubernetes deployment successful"
    
    # Get service information
    kubectl get services filepreview-rust-service
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    stop_service
    
    # Remove any test files
    rm -f test_image.png .service_pid
    
    print_success "Cleanup completed"
}

# Trap to ensure cleanup on script exit
trap cleanup EXIT

# Main function
main() {
    print_status "Starting File Preview Service build and test..."
    
    # Parse command line arguments
    SKIP_DEPS=false
    DOCKER_ONLY=false
    K8S_DEPLOY=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-deps)
                SKIP_DEPS=true
                shift
                ;;
            --docker-only)
                DOCKER_ONLY=true
                shift
                ;;
            --k8s)
                K8S_DEPLOY=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --skip-deps    Skip system dependency installation"
                echo "  --docker-only  Only build and test Docker container"
                echo "  --k8s          Deploy to Kubernetes"
                echo "  --help         Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Check OS compatibility
    check_os
    
    if [ "$DOCKER_ONLY" = true ]; then
        build_docker
        test_docker
        deploy_compose
        return 0
    fi
    
    if [ "$K8S_DEPLOY" = true ]; then
        build_docker
        deploy_k8s
        return 0
    fi
    
    # Install dependencies if not skipped
    if [ "$SKIP_DEPS" = false ]; then
        install_dependencies
    fi
    
    # Install Rust
    install_rust
    
    # Build and test
    build_project
    run_tests
    start_service
    test_endpoints
    test_sample_preview
    
    print_success "All tests passed! Service is ready for production."
    print_status "You can now:"
    print_status "  - Build Docker image: ./build-and-test.sh --docker-only"
    print_status "  - Deploy to K8s: ./build-and-test.sh --k8s"
    print_status "  - Access the service at: http://localhost:3000"
}

# Run main function
main "$@"