import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';

// Custom metrics for MQTT publish endpoint
const errorRate = new Rate('mqtt_publish_errors');
const successfulRequests = new Counter('mqtt_publish_success');
const failedRequests = new Counter('mqtt_publish_failed');
const mqttPublishResponseTime = new Trend('mqtt_publish_response_time');

// Test configuration
export const options = {
  stages: [
    { duration: '10s', target: 10 },  // Ramp up to 10 users
    { duration: '20s', target: 20 },  // Ramp up to 20 users  
    { duration: '10s', target: 20 },  // Stay at 20 users
    { duration: '10s', target: 0 },   // Ramp down to 0 users
  ],
  thresholds: {
    mqtt_publish_response_time: ['p(95)<1000', 'p(99)<2000'],
    mqtt_publish_errors: ['rate<0.1'], // Less than 10% error rate
    http_req_failed: ['rate<0.1'],
  },
};

const BASE_URL = 'http://127.0.0.1:3000';

// Test message data
const testMessages = [
  {
    topic: 'test/performance',
    message: 'Performance test message 1',
    qos: 0
  },
  {
    topic: 'test/load',
    message: 'Load testing with k6',
    qos: 1
  },
  {
    topic: 'test/stress',
    message: 'Stress testing MQTT endpoint',
    qos: 0
  },
  {
    topic: 'sensors/temperature',
    message: JSON.stringify({ temperature: 23.5, unit: 'C', timestamp: Date.now() }),
    qos: 1
  },
  {
    topic: 'alerts/critical',
    message: JSON.stringify({ level: 'critical', message: 'System alert', timestamp: Date.now() }),
    qos: 2
  },
];

export function setup() {
  console.log('🚀 Starting MQTT /mqtt/publish endpoint performance test');

  // Verify server is running
  const response = http.get(`${BASE_URL}/`);
  if (response.status !== 200) {
    throw new Error('Server is not responding correctly');
  }

  console.log('✅ Server is ready for MQTT publish testing');
  return { testStartTime: Date.now() };
}

export default function () {
  // Select a random test message
  const messageData = testMessages[Math.floor(Math.random() * testMessages.length)];

  // Test MQTT publish endpoint
  const startTime = Date.now();
  const response = http.post(`${BASE_URL}/mqtt/publish`, JSON.stringify(messageData), {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { endpoint: 'mqtt_publish' }
  });
  const responseTime = Date.now() - startTime;

  // Record response time
  mqttPublishResponseTime.add(responseTime);

  // Check response
  const success = check(response, {
    'MQTT publish status is 200': (r) => r.status === 200,
    'MQTT publish response time OK': (r) => r.timings.duration < 1000,
    'MQTT publish response has body': (r) => r.body && r.body.length > 0,
    'MQTT publish response contains success': (r) => r.body && r.body.includes('success'),
  });

  if (success) {
    successfulRequests.add(1);
  } else {
    failedRequests.add(1);
    errorRate.add(1);
    console.log(`❌ MQTT publish failed: Status ${response.status}, Response: ${response.body}`);
  }

  // Random sleep between requests
  sleep(Math.random() * 1 + 0.5); // 0.5-1.5 seconds
}

export function teardown(data) {
  console.log('🏁 MQTT publish endpoint test completed');
  const testDuration = (Date.now() - data.testStartTime) / 1000;
  console.log(`📊 Test duration: ${testDuration.toFixed(1)}s`);
}

export function handleSummary(data) {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const testDate = new Date().toLocaleString();
  const reportId = `mqtt_test_${new Date().toISOString().slice(0, 10).replace(/-/g, '')}_${new Date().toTimeString().slice(0, 8).replace(/:/g, '')}`;

  // Calculate metrics using correct data structure
  const httpReqs = data.metrics.http_reqs?.values?.count || 0;
  const httpReqFailed = data.metrics.http_req_failed?.values?.count || 0;
  const httpSuccessRate = httpReqs > 0 ? (((httpReqs - httpReqFailed) / httpReqs) * 100).toFixed(2) : '0';

  // Generate comprehensive report data
  const performanceData = {
    testInfo: {
      endpoint: '/mqtt/publish',
      timestamp: timestamp,
      testDate: testDate,
      duration: (data.state.testRunDurationMs / 1000).toFixed(2),
      iterations: data.metrics.iterations?.values?.count || 0,
      maxVus: data.metrics.vus_max?.values?.max || 0,
      totalRequests: httpReqs,
      successfulRequests: httpReqs - httpReqFailed,
      failedRequests: httpReqFailed,
      successRate: httpSuccessRate
    },
    metrics: {
      responseTime: {
        avg: data.metrics.http_req_duration?.values?.avg?.toFixed(2) || 0,
        min: data.metrics.http_req_duration?.values?.min?.toFixed(2) || 0,
        max: data.metrics.http_req_duration?.values?.max?.toFixed(2) || 0,
        p50: data.metrics.http_req_duration?.values?.p50?.toFixed(2) || 0,
        p90: data.metrics.http_req_duration?.values?.p90?.toFixed(2) || 0,
        p95: data.metrics.http_req_duration?.values?.p95?.toFixed(2) || 0,
        p99: data.metrics.http_req_duration?.values?.p99?.toFixed(2) || 0
      },
      dataTransfer: {
        received: (data.metrics.data_received?.values?.count / 1024)?.toFixed(2) || 0,
        sent: (data.metrics.data_sent?.values?.count / 1024)?.toFixed(2) || 0
      },
      errors: {
        rate: ((httpReqFailed / httpReqs) * 100)?.toFixed(2) || 0,
        count: httpReqFailed
      }
    },
    thresholds: {
      'p95_under_1000ms': data.metrics.http_req_duration?.values?.p95 < 1000,
      'p99_under_2000ms': data.metrics.http_req_duration?.values?.p99 < 2000,
      'error_rate_under_10pct': (httpReqFailed / httpReqs) < 0.1
    }
  };

  // Create HTML report
  const htmlReport = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MQTT Performance Test Report - ${testDate}</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
            overflow: hidden;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }
        .header h1 { font-size: 2.5em; margin-bottom: 10px; }
        .header .subtitle { font-size: 1.2em; opacity: 0.9; }
        .content { padding: 40px; }
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }
        .metric-card {
            background: #f8f9fa;
            padding: 25px;
            border-radius: 10px;
            border-left: 4px solid #667eea;
            transition: transform 0.2s;
        }
        .metric-card:hover { transform: translateY(-2px); }
        .metric-card h3 { color: #333; margin-bottom: 15px; font-size: 1.1em; }
        .metric-value {
            font-size: 2em;
            font-weight: bold;
            color: #667eea;
            margin-bottom: 5px;
        }
        .metric-label { color: #666; font-size: 0.9em; }
        .status-indicator {
            display: inline-block;
            padding: 5px 12px;
            border-radius: 20px;
            font-size: 0.8em;
            font-weight: bold;
            margin-left: 10px;
        }
        .status-pass {
            background: #d4edda;
            color: #155724;
        }
        .status-fail {
            background: #f8d7da;
            color: #721c24;
        }
        .response-times {
            background: #f8f9fa;
            padding: 30px;
            border-radius: 10px;
            margin-bottom: 30px;
        }
        .response-times h3 {
            color: #333;
            margin-bottom: 20px;
            font-size: 1.3em;
        }
        .response-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
            gap: 15px;
        }
        .response-item {
            text-align: center;
            padding: 15px;
            background: white;
            border-radius: 8px;
            border: 1px solid #e9ecef;
        }
        .response-item .value {
            font-size: 1.5em;
            font-weight: bold;
            color: #667eea;
            margin-bottom: 5px;
        }
        .response-item .label {
            color: #666;
            font-size: 0.85em;
        }
        .footer {
            background: #f8f9fa;
            padding: 20px;
            text-align: center;
            color: #666;
            font-size: 0.9em;
        }
        @media (max-width: 768px) {
            .container { margin: 10px; }
            .header { padding: 20px; }
            .header h1 { font-size: 2em; }
            .content { padding: 20px; }
            .metrics-grid { grid-template-columns: 1fr; }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎯 MQTT Performance Test Report</h1>
            <div class="subtitle">Endpoint: /mqtt/publish | ${testDate}</div>
        </div>
        
        <div class="content">
            <div class="metrics-grid">
                <div class="metric-card">
                    <h3>📊 Total Requests</h3>
                    <div class="metric-value">${performanceData.testInfo.totalRequests}</div>
                    <div class="metric-label">HTTP requests sent</div>
                </div>
                
                <div class="metric-card">
                    <h3>✅ Success Rate</h3>
                    <div class="metric-value">${performanceData.testInfo.successRate}%</div>
                    <div class="metric-label">${performanceData.testInfo.successfulRequests} successful</div>
                </div>
                
                <div class="metric-card">
                    <h3>⚡ Avg Response Time</h3>
                    <div class="metric-value">${performanceData.metrics.responseTime.avg}ms</div>
                    <div class="metric-label">Average latency</div>
                </div>
                
                <div class="metric-card">
                    <h3>👥 Max Virtual Users</h3>
                    <div class="metric-value">${performanceData.testInfo.maxVus}</div>
                    <div class="metric-label">Peak concurrent users</div>
                </div>
                
                <div class="metric-card">
                    <h3>⏱️ Test Duration</h3>
                    <div class="metric-value">${performanceData.testInfo.duration}s</div>
                    <div class="metric-label">Total test time</div>
                </div>
                
                <div class="metric-card">
                    <h3>❌ Error Rate</h3>
                    <div class="metric-value">${performanceData.metrics.errors.rate}%</div>
                    <div class="metric-label">${performanceData.metrics.errors.count} failed requests</div>
                </div>
            </div>
            
            <div class="response-times">
                <h3>📈 Response Time Distribution</h3>
                <div class="response-grid">
                    <div class="response-item">
                        <div class="value">${performanceData.metrics.responseTime.min}ms</div>
                        <div class="label">Minimum</div>
                    </div>
                    <div class="response-item">
                        <div class="value">${performanceData.metrics.responseTime.p50}ms</div>
                        <div class="label">P50 (Median)</div>
                    </div>
                    <div class="response-item">
                        <div class="value">${performanceData.metrics.responseTime.p90}ms</div>
                        <div class="label">P90</div>
                    </div>
                    <div class="response-item">
                        <div class="value">${performanceData.metrics.responseTime.p95}ms</div>
                        <div class="label">P95</div>
                    </div>
                    <div class="response-item">
                        <div class="value">${performanceData.metrics.responseTime.p99}ms</div>
                        <div class="label">P99</div>
                    </div>
                    <div class="response-item">
                        <div class="value">${performanceData.metrics.responseTime.max}ms</div>
                        <div class="label">Maximum</div>
                    </div>
                </div>
            </div>
            
            <div class="response-times">
                <h3>🎯 Threshold Results</h3>
                <div style="font-size: 1.1em; line-height: 1.8;">
                    <div>P95 Response Time &lt; 1000ms <span class="status-indicator ${performanceData.thresholds.p95_under_1000ms ? 'status-pass' : 'status-fail'}">${performanceData.thresholds.p95_under_1000ms ? '✅ PASS' : '❌ FAIL'}</span></div>
                    <div>P99 Response Time &lt; 2000ms <span class="status-indicator ${performanceData.thresholds.p99_under_2000ms ? 'status-pass' : 'status-fail'}">${performanceData.thresholds.p99_under_2000ms ? '✅ PASS' : '❌ FAIL'}</span></div>
                    <div>Error Rate &lt; 10% <span class="status-indicator ${performanceData.thresholds.error_rate_under_10pct ? 'status-pass' : 'status-fail'}">${performanceData.thresholds.error_rate_under_10pct ? '✅ PASS' : '❌ FAIL'}</span></div>
                </div>
            </div>
        </div>
        
        <div class="footer">
            Report generated on ${testDate} • Data Transfer: ${performanceData.metrics.dataTransfer.received}KB received, ${performanceData.metrics.dataTransfer.sent}KB sent
        </div>
    </div>
</body>
</html>`;

  // Console summary
  console.log(`\n🎯 MQTT Performance Test Results`);
  console.log(`════════════════════════════════════════`);
  console.log(`📊 Total Requests: ${performanceData.testInfo.totalRequests}`);
  console.log(`✅ Success Rate: ${performanceData.testInfo.successRate}%`);
  console.log(`⚡ Avg Response Time: ${performanceData.metrics.responseTime.avg}ms`);
  console.log(`📈 P95 Response Time: ${performanceData.metrics.responseTime.p95}ms`);
  console.log(`👥 Max Virtual Users: ${performanceData.testInfo.maxVus}`);
  console.log(`⏱️ Test Duration: ${performanceData.testInfo.duration}s`);
  console.log(`\n📁 Reports saved as: ${reportId}-report.html`);
  console.log(`🌐 Open the HTML file in your browser to view the interactive report!`);

  // Return files for k6 to save - fixed to save directly to current directory
  return {
    'stdout': `\n🎯 MQTT Test Complete!\n📊 ${httpReqs} requests, ${httpSuccessRate}% success\n⚡ ${performanceData.metrics.responseTime.avg}ms avg response time\n📁 Report: ${reportId}-report.html\n🌐 Open the HTML file in your browser!`,
    [`${reportId}-report.html`]: htmlReport,
    [`${reportId}-data.json`]: JSON.stringify(performanceData, null, 2),
    [`${reportId}-raw.json`]: JSON.stringify(data, null, 2)
  };
} 