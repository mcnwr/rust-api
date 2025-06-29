#!/bin/bash

# Comprehensive k6 Performance Test Runner with Coverage Reporting
# This script runs performance tests and generates detailed reports

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Function to print colored output
print_header() {
    echo -e "\n${PURPLE}=====================================${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}=====================================${NC}\n"
}

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

# Function to check if k6 is installed
check_k6() {
    if ! command -v k6 &> /dev/null; then
        print_error "k6 is not installed. Please install k6 and try again."
        print_status "Install k6 with: brew install k6 (macOS) or visit https://k6.io/docs/getting-started/installation/"
        exit 1
    fi
}

# Function to check if server is running
check_server() {
    print_status "Checking if server is running..."
    
    if curl -s -f http://127.0.0.1:3000/ > /dev/null 2>&1; then
        print_success "Server is running and responding"
        return 0
    else
        print_warning "Server is not responding. Checking Docker containers..."
        
        # Check if Docker containers are running
        if docker-compose ps | grep -q "Up"; then
            print_status "Docker containers are running, waiting for server to be ready..."
            sleep 10
            
            # Try again
            if curl -s -f http://127.0.0.1:3000/ > /dev/null 2>&1; then
                print_success "Server is now responding"
                return 0
            else
                print_error "Server is still not responding"
                return 1
            fi
        else
            print_error "Docker containers are not running"
            return 1
        fi
    fi
}

# Function to run performance tests
run_performance_test() {
    local test_type=$1
    local test_file=$2
    
    print_header "Running $test_type Performance Test"
    
    # Create timestamped report directory
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local report_dir="reports/${test_type}_${timestamp}"
    mkdir -p "$report_dir"
    
    print_status "Test reports will be saved in: $report_dir"
    
    # Run k6 test with custom report directory
    export K6_REPORT_DIR="$report_dir"
    
    if k6 run "$test_file" --out json="$report_dir/raw-results.json"; then
        print_success "$test_type test completed successfully"
        
        # Generate additional reports
        generate_custom_reports "$report_dir"
        
        return 0
    else
        print_error "$test_type test failed"
        return 1
    fi
}

# Function to generate custom reports
generate_custom_reports() {
    local report_dir=$1
    
    print_status "Generating additional custom reports..."
    
    # Create a comprehensive test summary
    cat > "$report_dir/test-summary.md" << EOF
# Performance Test Summary Report

**Test Date:** $(date)
**Test Type:** Load Testing with Coverage Analysis
**Target:** Rust API (Resource Constrained: 1 CPU, 512MB RAM)

## Test Configuration
- **Max Virtual Users:** 100
- **Test Duration:** 50 seconds (30s ramp-up + 10s sustained + 10s ramp-down)
- **Endpoints Tested:** 7 different API endpoints
- **Resource Limits:** Docker containers with 1 CPU core and 512MB RAM

## Generated Reports
1. \`performance-report.html\` - Interactive HTML report with charts
2. \`performance-summary.txt\` - Text summary of all metrics
3. \`performance-data.json\` - Raw test data in JSON format
4. \`endpoint-coverage.json\` - Detailed endpoint coverage analysis
5. \`raw-results.json\` - Raw k6 output data
6. \`docker-resources.txt\` - Resource usage during test
7. \`test-summary.md\` - This summary report

## How to View Reports
- Open \`performance-report.html\` in a web browser for interactive analysis
- Review \`endpoint-coverage.json\` for API coverage statistics
- Check \`docker-resources.txt\` for resource constraint validation

## Resource Monitoring
EOF

    # Capture current Docker resource usage
    if docker stats --no-stream > "$report_dir/docker-resources.txt" 2>/dev/null; then
        print_success "Docker resource usage captured"
        echo "Resource usage during test captured in docker-resources.txt" >> "$report_dir/test-summary.md"
    else
        print_warning "Could not capture Docker resource usage"
    fi
    
    # Create a quick analysis script
    cat > "$report_dir/analyze-results.py" << 'EOF'
#!/usr/bin/env python3
"""
Quick analysis script for k6 performance test results
"""
import json
import sys
from datetime import datetime

def analyze_coverage(coverage_file):
    try:
        with open(coverage_file, 'r') as f:
            data = json.load(f)
        
        print("📊 ENDPOINT COVERAGE ANALYSIS")
        print("=" * 50)
        
        summary = data.get('summary', {})
        print(f"Total Endpoints: {summary.get('totalEndpoints', 'N/A')}")
        print(f"Tested Endpoints: {summary.get('testedEndpoints', 'N/A')}")
        print(f"Coverage: {summary.get('coveragePercentage', 'N/A')}%")
        print()
        
        print("📈 ENDPOINT DETAILS")
        print("-" * 50)
        
        coverage = data.get('endpointCoverage', {})
        for endpoint, stats in coverage.items():
            status = "✅ TESTED" if stats.get('tested') else "❌ NOT TESTED"
            hits = stats.get('hits', 0)
            success_rate = stats.get('successRate', '0')
            avg_time = stats.get('avgResponseTime', '0')
            
            print(f"{endpoint:25} {status:12} Hits: {hits:4} Success: {success_rate:6}% Avg: {avg_time:8}")
        
    except FileNotFoundError:
        print("Coverage file not found")
    except json.JSONDecodeError:
        print("Invalid JSON in coverage file")

if __name__ == "__main__":
    coverage_file = "endpoint-coverage.json"
    if len(sys.argv) > 1:
        coverage_file = sys.argv[1]
    
    analyze_coverage(coverage_file)
EOF

    chmod +x "$report_dir/analyze-results.py"
    
    # Run the analysis if coverage file exists
    if [ -f "$report_dir/endpoint-coverage.json" ]; then
        print_status "Running coverage analysis..."
        python3 "$report_dir/analyze-results.py" "$report_dir/endpoint-coverage.json" > "$report_dir/coverage-analysis.txt"
        print_success "Coverage analysis completed"
    fi
    
    print_success "Custom reports generated in: $report_dir"
}

# Function to display test results
display_results() {
    local report_dir=$1
    
    print_header "Test Results Summary"
    
    # Find the most recent report directory if not specified
    if [ -z "$report_dir" ]; then
        report_dir=$(find reports -type d -name "*_*" | sort | tail -1)
    fi
    
    if [ -z "$report_dir" ] || [ ! -d "$report_dir" ]; then
        print_error "No test results found"
        return 1
    fi
    
    print_status "Latest test results: $report_dir"
    
    # Display coverage analysis if available
    if [ -f "$report_dir/coverage-analysis.txt" ]; then
        echo ""
        cat "$report_dir/coverage-analysis.txt"
        echo ""
    fi
    
    # List all generated files
    print_status "Generated files:"
    ls -la "$report_dir" | grep -v "^d" | awk '{print "  " $9 " (" $5 " bytes)"}'
    
    # Show HTML report location
    if [ -f "$report_dir/performance-report.html" ]; then
        local html_path="$(pwd)/$report_dir/performance-report.html"
        print_success "Interactive HTML report available at:"
        echo "  file://$html_path"
        print_status "Open this file in your web browser for detailed analysis"
    fi
}

# Main execution
main() {
    local test_type=${1:-"comprehensive"}
    local skip_docker_check=${2:-false}
    
    print_header "k6 Performance Test Runner with Coverage"
    
    # Check dependencies
    check_k6
    
    if [ "$skip_docker_check" != "true" ]; then
        check_docker
        
        # Check if server is running
        if ! check_server; then
            print_error "Server is not running. Please start the server first."
            print_status "Try running: docker-compose up -d"
            exit 1
        fi
    fi
    
    # Run the appropriate test
    case $test_type in
        "comprehensive"|"coverage")
            if run_performance_test "comprehensive" "k6-performance-test-with-reports.js"; then
                display_results
            else
                exit 1
            fi
            ;;
        "basic"|"simple")
            if run_performance_test "basic" "k6-performance-test.js"; then
                display_results
            else
                exit 1
            fi
            ;;
        "results"|"show")
            display_results
            ;;
        *)
            print_error "Unknown test type: $test_type"
            echo "Usage: $0 [comprehensive|basic|results] [skip-docker-check]"
            echo ""
            echo "Test types:"
            echo "  comprehensive  - Full test with coverage reporting (default)"
            echo "  basic         - Simple test without coverage"
            echo "  results       - Show latest test results"
            echo ""
            echo "Options:"
            echo "  skip-docker-check - Skip Docker and server checks"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@" 