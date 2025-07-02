# Rust API Performance Testing with k6

A high-performance Rust API with comprehensive k6 performance testing, endpoint coverage analysis, and beautiful web-based reporting. This project includes resource-constrained testing (1 CPU core, 512MB RAM) to simulate real-world deployment scenarios.

## 🚀 Quick Start

### 1. Prerequisites

Make sure you have the following installed:

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [k6](https://k6.io/docs/getting-started/installation/) for performance testing
- [Git](https://git-scm.com/downloads) for version control

```bash
# Install k6 on macOS
brew install k6

# Install k6 on Linux
sudo apt update && sudo apt install k6

# Install k6 on Windows
choco install k6
```

### 2. Quick Setup Verification

For a fast setup check, run our verification script:

```bash
# Quick setup test (30 seconds)
./test-setup.sh

# Full setup verification (2-3 minutes)
./verify-setup.sh
```

### 3. Start the Application

```bash
# Clone the repository (if needed)
git clone <your-repo-url>
cd rust-api

# Start the application with Docker
docker-compose up -d

# Wait for services to be ready (about 30 seconds)
sleep 30

# Verify the API is running
curl http://localhost:3000/
```

### 4. Run Performance Tests

```bash
# Run comprehensive performance test with coverage analysis
./run-performance-tests.sh

# Run basic performance test (faster, less detailed)
./run-performance-tests.sh basic

# View the latest test results
./run-performance-tests.sh results
```

### 5. View Results

After running tests, you have multiple ways to view results:

#### Option A: Web-Based Viewer (Recommended)

```bash
# Open the performance dashboard in your browser
open http://localhost:3000/reports
```

#### Option B: HTML Reports

```bash
# Open the latest HTML report
open reports/*/performance-report.html
```

#### Option C: Command Line

```bash
# View latest results summary
./run-performance-tests.sh results
```

## 📊 Understanding Your Test Results

### Performance Metrics Explained

| Metric                | What It Means                              | Good Target        |
| --------------------- | ------------------------------------------ | ------------------ |
| **Response Time P95** | 95% of requests completed within this time | < 500ms            |
| **Response Time P99** | 99% of requests completed within this time | < 1000ms           |
| **Error Rate**        | Percentage of failed requests              | < 5%               |
| **Requests/sec**      | Throughput under load                      | Varies by endpoint |
| **Endpoint Coverage** | Percentage of API endpoints tested         | > 85%              |

### Sample Good Results

```
✅ Test Results Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Overall Performance
   Total Requests: 2,847
   Success Rate: 97.8%
   Error Rate: 2.2%
   Requests/sec: 56.9

⏱️  Response Times
   Average: 45ms
   P95: 198ms
   P99: 456ms

🎯 Endpoint Coverage: 100% (7/7 endpoints tested)
```

### Warning Signs to Watch For

🚨 **High Error Rates (>5%)**

- System may be overloaded
- Check Docker container resources
- Consider reducing load or scaling up

🚨 **Slow Response Times (P95 >500ms)**

- Performance degradation under load
- May need optimization or more resources

🚨 **Low Coverage (<50%)**

- Not all endpoints being tested
- Review test scenarios in k6 scripts

## 🎯 Available API Endpoints

Your Rust API includes these endpoints for testing:

| Endpoint          | Method | Purpose               | Expected Response Time |
| ----------------- | ------ | --------------------- | ---------------------- |
| `/`               | GET    | Health check          | < 50ms                 |
| `/user/users`     | GET    | List all users        | < 200ms                |
| `/user/users`     | POST   | Create new user       | < 300ms                |
| `/user/users/:id` | GET    | Get specific user     | < 200ms                |
| `/mqtt/pub`       | POST   | Publish MQTT message  | < 500ms                |
| `/mqtt/consume`   | GET    | Consume MQTT messages | < 500ms                |
| `/channel/pub`    | POST   | Publish to channel    | < 400ms                |

## 📁 Test Reports Structure

Each test run creates a timestamped directory in `reports/` with comprehensive results:

```
reports/
└── comprehensive_20241219_143022/
    ├── 📊 performance-report.html      # Interactive charts & graphs
    ├── 📋 performance-summary.txt      # Quick text summary
    ├── 🎯 endpoint-coverage.json       # Coverage analysis
    ├── 📄 performance-data.json        # Structured metrics
    ├── 🔧 raw-results.json            # Raw k6 output
    ├── 🐳 docker-resources.txt        # Resource usage
    ├── 📝 test-summary.md             # Test configuration info
    └── 🐍 analyze-results.py          # Quick analysis script
```

### How to Use Each Report

#### 1. Interactive HTML Report (`performance-report.html`)

- **Best for**: Visual analysis and presentations
- **Contains**: Charts, graphs, timeline analysis
- **How to use**: Open in any web browser

#### 2. Coverage Analysis (`endpoint-coverage.json`)

```json
{
  "summary": {
    "totalEndpoints": 7,
    "testedEndpoints": 7,
    "coveragePercentage": "100.00"
  },
  "endpointCoverage": {
    "GET /": {
      "hits": 45,
      "successRate": "97.78",
      "avgResponseTime": "12.50",
      "tested": true
    }
  }
}
```

#### 3. Quick Analysis Script

```bash
# Run the auto-generated analysis
cd reports/latest_test_directory
python3 analyze-results.py endpoint-coverage.json
```

## 🔧 Advanced Testing Options

### Custom Test Duration

```javascript
// Edit k6-performance-test-with-reports.js
export const options = {
  stages: [
    { duration: "60s", target: 150 }, // 60 seconds to 150 users
    { duration: "30s", target: 150 }, // Hold 150 users for 30 seconds
    { duration: "30s", target: 0 }, // Ramp down over 30 seconds
  ],
};
```

### Environment Variables

```bash
# For k6 Cloud integration
export K6_PROJECT_ID=123

# Custom report directory
export K6_REPORT_DIR=custom-reports

# Run with custom config
./run-performance-tests.sh comprehensive
```

### Resource Constraint Testing

Your setup includes Docker resource limits:

- **CPU**: 1 core maximum
- **Memory**: 512MB maximum
- **Purpose**: Simulate production constraints

```yaml
# docker-compose.yml
deploy:
  resources:
    limits:
      cpus: "1.0"
      memory: 512M
```

## 🌐 Web-Based Performance Dashboard

Your Rust API includes a built-in performance viewer:

### Dashboard Features

- 🏠 **Overview**: Quick stats and recent reports
- 📊 **Report Browser**: Filter and sort all test results
- 📈 **Detailed Analysis**: Interactive charts and metrics
- 🎯 **Coverage Analysis**: Endpoint testing coverage
- 📄 **Raw Data**: JSON data inspection
- 🔄 **Auto-refresh**: Real-time updates

### Accessing the Dashboard

```bash
# Make sure your API is running
docker-compose up -d

# Open dashboard pages
open http://localhost:3000/           # Home dashboard
open http://localhost:3000/reports    # All reports
open http://localhost:3000/reports/latest  # Latest report
```

### API Endpoints for Programmatic Access

```bash
# Get all reports as JSON
curl http://localhost:3000/api/reports

# Get specific report data
curl http://localhost:3000/api/reports/{report_id}
```

## 🐛 Troubleshooting Guide

### 1. Server Not Responding

```bash
# Check Docker containers
docker-compose ps

# Check logs
docker-compose logs rust-api

# Restart services
docker-compose down && docker-compose up -d

# Wait for startup
sleep 30
```

### 2. k6 Installation Issues

```bash
# macOS with Homebrew
brew install k6

# Linux (Ubuntu/Debian)
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# Windows with Chocolatey
choco install k6

# Direct download
# Visit https://k6.io/docs/getting-started/installation/
```

### 3. No Reports Generated

```bash
# Check if reports directory exists
ls -la reports/

# Run test with verbose output
./run-performance-tests.sh comprehensive 2>&1 | tee test-debug.log

# Check permissions
chmod +x run-performance-tests.sh
```

### 4. High Error Rates

```bash
# Check system resources
docker stats

# Increase warmup time
# Edit k6 script to add longer ramp-up period

# Check individual endpoints
curl -w "Time: %{time_total}s\n" http://localhost:3000/
```

### 5. Performance Dashboard Not Loading

```bash
# Verify Rust API is running
curl http://localhost:3000/

# Check templates directory
ls -la templates/

# Restart with logs
docker-compose up --build
```

## 📈 Performance Benchmarks

### Expected Results (1 CPU, 512MB RAM)

| Endpoint           | Success Rate | Avg Response Time | P95 Response Time |
| ------------------ | ------------ | ----------------- | ----------------- |
| Health Check (`/`) | 95%+         | < 50ms            | < 100ms           |
| User Operations    | 80%+         | < 200ms           | < 400ms           |
| MQTT Operations    | 60%+         | < 500ms           | < 800ms           |
| Channel Operations | 60%+         | < 400ms           | < 600ms           |

### Resource Utilization Targets

- **CPU**: Should reach ~100% during peak load
- **Memory**: Should stay under 512MB limit
- **Test Duration**: ~50 seconds total
- **Virtual Users**: Up to 100 concurrent

## 🔄 Continuous Integration

### GitHub Actions Example

```yaml
name: Performance Tests
on: [push, pull_request]
jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install k6
        run: |
          sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6
      - name: Start services
        run: docker-compose up -d
      - name: Run performance tests
        run: ./run-performance-tests.sh comprehensive
      - name: Upload reports
        uses: actions/upload-artifact@v3
        with:
          name: performance-reports
          path: reports/
```

## 📚 Additional Resources

- **k6 Documentation**: https://k6.io/docs/
- **Docker Compose**: https://docs.docker.com/compose/
- **Rust Performance**: https://rust-lang.github.io/book/
- **Load Testing Best Practices**: https://k6.io/docs/testing-guides/

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add or modify tests as needed
4. Run performance tests to ensure no regressions
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Last Updated**: $(date)  
**API Framework**: Rust + Axum  
**Testing Framework**: k6  
**Resource Limits**: 1 CPU Core, 512MB RAM  
**Coverage**: 7 API endpoints
