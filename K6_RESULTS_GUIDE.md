# k6 Performance Test Results Guide

This guide explains how to read, interpret, and act on your k6 performance test results for the Rust API.

## 🎯 Step-by-Step: Running Your First Test

### 1. Run the Test

```bash
# Start services
docker-compose up -d
sleep 30

# Run comprehensive test
./run-performance-tests.sh

# Wait for completion (about 2-3 minutes)
```

### 2. Understanding the Console Output

When k6 runs, you'll see output like this:

```
     ✓ health check status is 200
     ✓ get users status is 200
     ✓ create user status is 200 or 201

     checks.........................: 97.78% ✓ 2847    ✗ 65
     data_received..................: 1.2 MB 24 kB/s
     data_sent......................: 890 kB 18 kB/s
     http_req_blocked...............: avg=1.2ms    min=1µs      med=8µs      max=45ms     p(90)=15µs   p(95)=28µs
     http_req_connecting............: avg=580µs    min=0s       med=0s       max=12ms     p(90)=0s     p(95)=2ms
     http_req_duration..............: avg=45ms     min=1.2ms    med=28ms     max=890ms    p(90)=95ms   p(95)=198ms
     http_req_failed................: 2.22% ✓ 65      ✗ 2847
     http_req_receiving.............: avg=120µs    min=18µs     med=95µs     max=2.1ms    p(90)=189µs  p(95)=245µs
     http_req_sending...............: avg=45µs     min=8µs      med=35µs     max=890µs    p(90)=78µs   p(95)=120µs
     http_req_waiting...............: avg=44ms     min=1.1ms    med=27ms     max=887ms    p(90)=94ms   p(95)=197ms
     http_reqs......................: 2912 58.24/s
     iteration_duration.............: avg=1.7s     min=1.0s     med=1.6s     max=3.4s     p(90)=2.3s   p(95)=2.8s
     iterations.....................: 1456 29.12/s
     vus............................: 100 min=0     max=100
     vus_max........................: 100
```

### 3. Key Metrics Explained

#### ✅ **Checks (Success Rate)**

```
checks.........................: 97.78% ✓ 2847    ✗ 65
```

- **What it means**: 97.78% of all test assertions passed
- **Good**: > 95%
- **Warning**: < 90%
- **Action if low**: Check endpoint errors, server logs

#### ⏱️ **Response Times**

```
http_req_duration..............: avg=45ms     min=1.2ms    med=28ms     max=890ms    p(90)=95ms   p(95)=198ms
```

- **avg=45ms**: Average response time across all requests
- **p(95)=198ms**: 95% of requests completed within 198ms
- **max=890ms**: Slowest request took 890ms

**Targets by endpoint:**

- Health check: avg < 50ms, p95 < 100ms
- User operations: avg < 200ms, p95 < 400ms
- MQTT operations: avg < 500ms, p95 < 800ms

#### ❌ **Error Rate**

```
http_req_failed................: 2.22% ✓ 65      ✗ 2847
```

- **2.22%**: 2.22% of requests failed
- **Good**: < 5%
- **Warning**: > 5%
- **Critical**: > 10%

#### 🚀 **Throughput**

```
http_reqs......................: 2912 58.24/s
```

- **2912 total requests** in the test
- **58.24 requests/second** average throughput

## 📊 Reading the Generated Reports

### 1. Web Dashboard (Recommended)

Open http://localhost:3000/reports to see:

#### Dashboard Overview

```
📊 Latest Performance Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Report: comprehensive_20241219_143022
Status: ✅ PASSED
Duration: 50.2 seconds
Total Requests: 2,912
Success Rate: 97.78%
Avg Response Time: 45ms
```

#### Endpoint Coverage Analysis

```
🎯 Endpoint Coverage: 100% (7/7 endpoints)

GET /                  ✅ TESTED    Hits: 458   Success: 98.9%   Avg: 12ms
GET /user/users        ✅ TESTED    Hits: 367   Success: 96.2%   Avg: 85ms
POST /user/users       ✅ TESTED    Hits: 321   Success: 94.1%   Avg: 145ms
GET /user/users/:id    ✅ TESTED    Hits: 298   Success: 97.3%   Avg: 78ms
POST /mqtt/pub         ✅ TESTED    Hits: 287   Success: 89.2%   Avg: 234ms
GET /mqtt/consume      ✅ TESTED    Hits: 312   Success: 91.7%   Avg: 189ms
POST /channel/pub      ✅ TESTED    Hits: 269   Success: 88.8%   Avg: 201ms
```

### 2. HTML Report (`performance-report.html`)

Open with any browser to see:

- **Timeline charts** showing performance over time
- **Response time distribution** graphs
- **Error rate visualization**
- **Virtual user ramping** graphs

### 3. JSON Files for Analysis

#### `endpoint-coverage.json`

```json
{
  "summary": {
    "totalEndpoints": 7,
    "testedEndpoints": 7,
    "coveragePercentage": "100.00",
    "testDate": "2024-12-19T14:30:22Z",
    "testDuration": "50.2s"
  },
  "endpointCoverage": {
    "GET /": {
      "hits": 458,
      "success": 453,
      "errors": 5,
      "successRate": "98.91",
      "avgResponseTime": "12.50",
      "tested": true
    }
  }
}
```

## 🚨 Warning Signs & Solutions

### High Error Rates (>5%)

**Example bad result:**

```
http_req_failed................: 15.5% ✓ 450     ✗ 2462
```

**Possible causes:**

- Server overloaded
- Database connection issues
- Resource constraints hit

**Solutions:**

```bash
# Check resource usage
docker stats

# Check logs for errors
docker-compose logs rust-api | tail -50

# Reduce test load
# Edit k6-performance-test-with-reports.js
# Change: { duration: '30s', target: 50 }  // Reduced from 100
```

### Slow Response Times

**Example bad result:**

```
http_req_duration..............: avg=750ms    p(95)=1.2s    max=5.8s
```

**Analysis:**

- Average 750ms is too slow for most endpoints
- P95 of 1.2s means 5% of users waited over 1.2 seconds
- Max 5.8s indicates severe performance spikes

**Solutions:**

```bash
# Check if services are healthy
curl -w "Total time: %{time_total}s\n" http://localhost:3000/

# Monitor during test
docker stats --no-stream

# Optimize queries or increase resources
```

### Low Coverage

**Example bad result:**

```
🎯 Endpoint Coverage: 43% (3/7 endpoints)

GET /                  ✅ TESTED    Hits: 458   Success: 98.9%
GET /user/users        ❌ NOT TESTED Hits: 0     Success: 0%
POST /user/users       ❌ NOT TESTED Hits: 0     Success: 0%
```

**Solution:** Update test scenarios in `k6-performance-test-with-reports.js`

## 📈 Understanding Performance Trends

### Good Performance Progression

```
Test 1: avg=45ms,  p95=198ms,  errors=2.2%  ✅
Test 2: avg=42ms,  p95=187ms,  errors=1.8%  ✅ Improving
Test 3: avg=38ms,  p95=165ms,  errors=1.1%  ✅ Great!
```

### Performance Regression

```
Test 1: avg=45ms,  p95=198ms,  errors=2.2%  ✅
Test 2: avg=67ms,  p95=245ms,  errors=4.1%  ⚠️  Regression
Test 3: avg=89ms,  p95=320ms,  errors=7.8%  🚨 Critical
```

## 🔍 Advanced Analysis

### Using the Python Analysis Script

Each test generates an analysis script:

```bash
cd reports/latest_test_directory
python3 analyze-results.py endpoint-coverage.json
```

Output:

```
📊 ENDPOINT COVERAGE ANALYSIS
==================================================
Total Endpoints: 7
Tested Endpoints: 7
Coverage: 100.00%

📈 ENDPOINT DETAILS
--------------------------------------------------
GET /                     ✅ TESTED      Hits:  458 Success:  98.9% Avg:    12ms
GET /user/users           ✅ TESTED      Hits:  367 Success:  96.2% Avg:    85ms
POST /user/users          ✅ TESTED      Hits:  321 Success:  94.1% Avg:   145ms
```

### Comparing Multiple Test Runs

```bash
# Compare two test results
diff reports/test1/endpoint-coverage.json reports/test2/endpoint-coverage.json

# Extract key metrics for comparison
jq '.summary' reports/*/endpoint-coverage.json
```

### Setting Up Alerts

Create a simple script to alert on bad performance:

```bash
#!/bin/bash
# performance-alert.sh

LATEST_REPORT=$(ls -t reports/ | head -1)
ERROR_RATE=$(jq -r '.summary.errorRate // "0"' "reports/$LATEST_REPORT/endpoint-coverage.json")
AVG_RESPONSE=$(jq -r '.summary.avgResponseTime // "0"' "reports/$LATEST_REPORT/endpoint-coverage.json")

if (( $(echo "$ERROR_RATE > 0.05" | bc -l) )); then
    echo "🚨 ALERT: High error rate: $ERROR_RATE"
fi

if (( $(echo "$AVG_RESPONSE > 200" | bc -l) )); then
    echo "🚨 ALERT: Slow response time: ${AVG_RESPONSE}ms"
fi
```

## 📊 Performance Benchmarking

### Resource-Constrained Benchmarks (1 CPU, 512MB)

| Metric           | Good        | Warning   | Critical |
| ---------------- | ----------- | --------- | -------- |
| **Health Check** | < 50ms avg  | 50-100ms  | > 100ms  |
| **User Ops**     | < 200ms avg | 200-400ms | > 400ms  |
| **MQTT Ops**     | < 500ms avg | 500-800ms | > 800ms  |
| **Error Rate**   | < 2%        | 2-5%      | > 5%     |
| **Coverage**     | > 90%       | 70-90%    | < 70%    |

### Production Readiness Checklist

✅ **Performance**

- [ ] P95 response time < 500ms for all endpoints
- [ ] Error rate < 2% under sustained load
- [ ] Handles 100 concurrent users without degradation

✅ **Coverage**

- [ ] All critical endpoints tested (100% coverage)
- [ ] All CRUD operations verified
- [ ] Error scenarios tested

✅ **Reliability**

- [ ] No memory leaks detected during extended testing
- [ ] Graceful degradation under extreme load
- [ ] Quick recovery after load spikes

## 🎓 Next Steps

1. **Set up automated testing**: Add performance tests to CI/CD
2. **Create custom thresholds**: Adjust targets based on your requirements
3. **Monitor trends**: Track performance over time
4. **Optimize bottlenecks**: Focus on slowest endpoints first
5. **Scale testing**: Test with higher loads as needed

For more detailed information, see:

- [README.md](README.md) - Complete project documentation
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Common commands
- [PERFORMANCE_TESTING.md](PERFORMANCE_TESTING.md) - Advanced testing options
