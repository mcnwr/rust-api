#!/bin/bash

set -e

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

# Function to check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
}

# Function to build and run production
run_production() {
    print_status "Starting production environment with resource limits..."
    print_status "Resource limits: 1 CPU core, 512MB RAM for rust-api"
    
    docker-compose down
    docker-compose build --no-cache
    docker-compose up -d
    
    print_success "Production environment started!"
    print_status "Rust API: http://localhost:3000"
    print_status "RabbitMQ Management: http://localhost:15672 (guest/guest)"
}

# Function to run development
run_development() {
    print_status "Starting development environment..."
    
    docker-compose -f docker-compose.yml -f docker-compose.dev.yml down
    docker-compose -f docker-compose.yml -f docker-compose.dev.yml build
    docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d
    
    print_success "Development environment started!"
    print_status "Rust API: http://localhost:3000 (with hot reload)"
    print_status "RabbitMQ Management: http://localhost:15672 (admin/admin123)"
}

# Function to monitor resources
monitor_resources() {
    print_status "Monitoring resource usage..."
    while true; do
        clear
        echo -e "${BLUE}=== Docker Container Resource Usage ===${NC}"
        docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}"
        echo ""
        echo -e "${YELLOW}Press Ctrl+C to exit monitoring${NC}"
        sleep 2
    done
}

# Function to run k6 performance test
run_k6_test() {
    print_status "Running k6 performance test..."
    
    # Check if services are running
    if ! curl -s http://localhost:3000/ >/dev/null 2>&1; then
        print_error "Rust API is not responding. Make sure the services are running."
        exit 1
    fi
    
    if command -v k6 >/dev/null 2>&1; then
        k6 run k6-performance-test.js
    else
        print_warning "k6 not found. Running with Docker..."
        docker run --rm -i --network host grafana/k6:latest run - < k6-performance-test.js
    fi
}

# Function to show logs
show_logs() {
    local service=${1:-}
    if [ -n "$service" ]; then
        print_status "Showing logs for $service..."
        docker-compose logs -f "$service"
    else
        print_status "Showing logs for all services..."
        docker-compose logs -f
    fi
}

# Function to stop services
stop_services() {
    print_status "Stopping all services..."
    docker-compose down
    print_success "All services stopped!"
}

# Function to cleanup
cleanup() {
    print_status "Cleaning up Docker resources..."
    docker-compose down -v
    docker system prune -f
    print_success "Cleanup completed!"
}

# Main menu
show_help() {
    echo -e "${BLUE}Docker Management Script${NC}"
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  prod, production    - Start production environment (resource limited)"
    echo "  dev, development    - Start development environment (with hot reload)"
    echo "  monitor, stats      - Monitor resource usage in real-time"
    echo "  test, k6           - Run k6 performance test"
    echo "  logs [service]     - Show logs (optionally for specific service)"
    echo "  stop               - Stop all services" 
    echo "  clean, cleanup     - Stop services and cleanup Docker resources"
    echo "  help, -h, --help   - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 prod            # Start production with limits"
    echo "  $0 dev             # Start development mode"
    echo "  $0 monitor         # Monitor resource usage"
    echo "  $0 logs rust-api   # Show logs for rust-api service"
}

# Check Docker availability
check_docker

# Parse command line arguments
case "${1:-help}" in
    prod|production)
        run_production
        ;;
    dev|development)
        run_development
        ;;
    monitor|stats)
        monitor_resources
        ;;
    test|k6)
        run_k6_test
        ;;
    logs)
        show_logs "$2"
        ;;
    stop)
        stop_services
        ;;
    clean|cleanup)
        cleanup
        ;;
    help|-h|--help)
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac 