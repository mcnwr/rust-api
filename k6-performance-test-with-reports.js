import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';
import { htmlReport } from 'https://raw.githubusercontent.com/benc-uk/k6-reporter/main/dist/bundle.js';
import { textSummary } from 'https://jslib.k6.io/k6-summary/0.0.1/index.js';

// Custom metrics for coverage and detailed analysis
const errorRate = new Rate('errors');
const endpointHits = new Counter('endpoint_hits');
const endpointResponseTime = new Trend('endpoint_response_time');
const successfulRequests = new Counter('successful_requests');
const failedRequests = new Counter('failed_requests');

// Endpoint coverage tracking
const endpointCoverage = {
  'GET /': { hits: 0, success: 0, errors: 0, totalResponseTime: 0 },
  'GET /user/users': { hits: 0, success: 0, errors: 0, totalResponseTime: 0 },
  'POST /user/users': { hits: 0, success: 0, errors: 0, totalResponseTime: 0 },
  'GET /user/users/:id': { hits: 0, success: 0, errors: 0, totalResponseTime: 0 },
  'POST /mqtt/pub': { hits: 0, success: 0, errors: 0, totalResponseTime: 0 },
  'GET /mqtt/consume': { hits: 0, success: 0, errors: 0, totalResponseTime: 0 },
  'POST /channel/pub': { hits: 0, success: 0, errors: 0, totalResponseTime: 0 },
};

// Test configuration with reporting
export const options = {
  stages: [
    { duration: '30s', target: 100 }, // Ramp up to 100 users
    { duration: '10s', target: 100 }, // Stay at 100 users
    { duration: '10s', target: 0 },   // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1000'],
    http_req_failed: ['rate<0.05'],
    errors: ['rate<0.05'],
    successful_requests: ['count>100'],
    endpoint_hits: ['count>200'],
  },
  // External metrics outputs
  ext: {
    loadimpact: {
      projectID: parseInt(__ENV.K6_PROJECT_ID) || 0,
      name: 'Rust API Performance Test'
    }
  }
};

const BASE_URL = 'http://127.0.0.1:3000';

// Test data
const testUsers = [
  { name: 'Alice', email: 'alice@example.com' },
  { name: 'Bob', email: 'bob@example.com' },
  { name: 'Charlie', email: 'charlie@example.com' },
  { name: 'Diana', email: 'diana@example.com' },
  { name: 'Eve', email: 'eve@example.com' },
];

const testMessages = [
  { message: 'Hello World', priority: 'high' },
  { message: 'Test Message', priority: 'medium' },
  { message: 'Performance Test', priority: 'low' },
  { message: 'Load Test Data', priority: 'high' },
  { message: 'Coverage Test', priority: 'medium' },
];

// Helper function to track endpoint coverage
function trackEndpoint(endpoint, success, responseTime) {
  if (endpointCoverage[endpoint]) {
    endpointCoverage[endpoint].hits++;
    endpointCoverage[endpoint].totalResponseTime += responseTime;
    if (success) {
      endpointCoverage[endpoint].success++;
      successfulRequests.add(1);
    } else {
      endpointCoverage[endpoint].errors++;
      failedRequests.add(1);
    }
  }
  endpointHits.add(1);
  endpointResponseTime.add(responseTime);
}

export function setup() {
  console.log('🚀 Starting Performance Test with Coverage Reporting');

  // Verify server is running
  const response = http.get(`${BASE_URL}/`);
  if (response.status !== 200) {
    throw new Error('Server is not responding correctly');
  }
  console.log('✅ Server is ready for testing');
  console.log('📊 Coverage tracking enabled for 7 endpoints');

  return {
    serverReady: true,
    testStartTime: Date.now(),
    endpoints: Object.keys(endpointCoverage)
  };
}

export default function () {
  const testScenario = Math.random();

  if (testScenario < 0.25) {
    // 25% - Test basic health check
    testHealthCheck();
  } else if (testScenario < 0.5) {
    // 25% - Test user operations  
    testUserOperations();
  } else if (testScenario < 0.75) {
    // 25% - Test MQTT operations
    testMqttOperations();
  } else {
    // 25% - Test channel operations
    testChannelOperations();
  }

  sleep(Math.random() * 2 + 0.5); // Random sleep between 0.5-2.5 seconds
}

function testHealthCheck() {
  const startTime = Date.now();
  const response = http.get(`${BASE_URL}/`, {
    tags: { endpoint: 'health_check' }
  });
  const responseTime = Date.now() - startTime;

  const success = check(response, {
    'health check status is 200': (r) => r.status === 200,
    'health check response time OK': (r) => r.timings.duration < 100,
    'health check body contains Hello': (r) => r.body && r.body.includes('Hello'),
  });

  trackEndpoint('GET /', success, responseTime);

  if (!success) {
    errorRate.add(1);
  }
}

function testUserOperations() {
  // Test getting all users
  let startTime = Date.now();
  let response = http.get(`${BASE_URL}/user/users`, {
    tags: { endpoint: 'get_users' }
  });
  let responseTime = Date.now() - startTime;

  let success = check(response, {
    'get users status is 200': (r) => r.status === 200,
    'get users response time OK': (r) => r.timings.duration < 200,
  });

  trackEndpoint('GET /user/users', success, responseTime);

  if (!success) {
    errorRate.add(1);
    return;
  }

  // Test creating a user
  const userData = testUsers[Math.floor(Math.random() * testUsers.length)];
  startTime = Date.now();
  response = http.post(`${BASE_URL}/user/users`, JSON.stringify(userData), {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'create_user' }
  });
  responseTime = Date.now() - startTime;

  success = check(response, {
    'create user status is 200 or 201': (r) => r.status === 200 || r.status === 201,
    'create user response time OK': (r) => r.timings.duration < 300,
  });

  trackEndpoint('POST /user/users', success, responseTime);

  if (!success) {
    errorRate.add(1);
    return;
  }

  // Test getting a specific user
  startTime = Date.now();
  response = http.get(`${BASE_URL}/user/users/1`, {
    tags: { endpoint: 'get_user_by_id' }
  });
  responseTime = Date.now() - startTime;

  success = check(response, {
    'get specific user response time OK': (r) => r.timings.duration < 200,
  });

  trackEndpoint('GET /user/users/:id', success, responseTime);
}

function testMqttOperations() {
  // Test MQTT publisher
  const messageData = testMessages[Math.floor(Math.random() * testMessages.length)];
  let startTime = Date.now();
  let response = http.post(`${BASE_URL}/mqtt/pub`, JSON.stringify(messageData), {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'mqtt_publish' }
  });
  let responseTime = Date.now() - startTime;

  let success = check(response, {
    'mqtt publish status is 200': (r) => r.status === 200,
    'mqtt publish response time OK': (r) => r.timings.duration < 500,
  });

  trackEndpoint('POST /mqtt/pub', success, responseTime);

  if (!success) {
    errorRate.add(1);
    return;
  }

  // Test MQTT consumer
  startTime = Date.now();
  response = http.get(`${BASE_URL}/mqtt/consume`, {
    tags: { endpoint: 'mqtt_consume' }
  });
  responseTime = Date.now() - startTime;

  success = check(response, {
    'mqtt consume response time OK': (r) => r.timings.duration < 300,
  });

  trackEndpoint('GET /mqtt/consume', success, responseTime);

  if (!success) {
    errorRate.add(1);
  }
}

function testChannelOperations() {
  // Test channel publication
  const channelData = {
    user_id: Math.floor(Math.random() * 100) + 1,
    message: 'Channel test message - ' + Math.random().toString(36).substring(7),
    timestamp: new Date().toISOString(),
  };

  const startTime = Date.now();
  const response = http.post(`${BASE_URL}/channel/pub`, JSON.stringify(channelData), {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'channel_publish' }
  });
  const responseTime = Date.now() - startTime;

  const success = check(response, {
    'channel publish response time OK': (r) => r.timings.duration < 400,
    'channel publish status check': (r) => r.status >= 200 && r.status < 300,
  });

  trackEndpoint('POST /channel/pub', success, responseTime);

  if (!success) {
    errorRate.add(1);
  }
}

export function teardown(data) {
  console.log('🏁 Performance test completed');
  console.log('📊 Generating comprehensive reports...');

  // Calculate coverage statistics
  const coverageStats = {};
  let totalEndpoints = 0;
  let testedEndpoints = 0;

  for (const [endpoint, stats] of Object.entries(endpointCoverage)) {
    totalEndpoints++;
    if (stats.hits > 0) {
      testedEndpoints++;
    }

    coverageStats[endpoint] = {
      hits: stats.hits,
      successRate: stats.hits > 0 ? ((stats.success / stats.hits) * 100).toFixed(2) + '%' : '0%',
      avgResponseTime: stats.hits > 0 ? (stats.totalResponseTime / stats.hits).toFixed(2) + 'ms' : '0ms',
      errors: stats.errors
    };
  }

  const coveragePercentage = ((testedEndpoints / totalEndpoints) * 100).toFixed(2);

  console.log(`📈 Endpoint Coverage: ${testedEndpoints}/${totalEndpoints} (${coveragePercentage}%)`);
  console.log('📋 Detailed coverage report will be saved to files');

  return {
    testEndTime: Date.now(),
    testDuration: Date.now() - data.testStartTime,
    coverageStats,
    coveragePercentage,
    totalEndpoints,
    testedEndpoints
  };
}

export function handleSummary(data) {
  const testStartTime = new Date().toISOString().replace(/[:.]/g, '-');

  // Calculate endpoint coverage
  const coverageReport = {
    testInfo: {
      timestamp: testStartTime,
      duration: data.metrics.iteration_duration?.avg || 0,
      iterations: data.metrics.iterations?.count || 0,
      vus: data.metrics.vus_max?.value || 0
    },
    endpointCoverage: {}
  };

  // Generate coverage data
  for (const [endpoint, stats] of Object.entries(endpointCoverage)) {
    coverageReport.endpointCoverage[endpoint] = {
      hits: stats.hits,
      successRate: stats.hits > 0 ? ((stats.success / stats.hits) * 100).toFixed(2) : '0',
      avgResponseTime: stats.hits > 0 ? (stats.totalResponseTime / stats.hits).toFixed(2) : '0',
      errors: stats.errors,
      tested: stats.hits > 0
    };
  }

  const totalEndpoints = Object.keys(endpointCoverage).length;
  const testedEndpoints = Object.values(endpointCoverage).filter(stats => stats.hits > 0).length;
  coverageReport.summary = {
    totalEndpoints,
    testedEndpoints,
    coveragePercentage: ((testedEndpoints / totalEndpoints) * 100).toFixed(2)
  };

  return {
    'reports/performance-report.html': htmlReport(data, {
      title: 'Rust API Performance Test Report',
      description: 'Comprehensive performance analysis with endpoint coverage'
    }),
    'reports/performance-summary.txt': textSummary(data, { indent: ' ', enableColors: false }),
    'reports/performance-data.json': JSON.stringify(data, null, 2),
    'reports/endpoint-coverage.json': JSON.stringify(coverageReport, null, 2),
    stdout: textSummary(data, { indent: ' ', enableColors: true })
  };
} 