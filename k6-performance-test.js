import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');

// Test configuration
export const options = {
  stages: [
    // Load Test - Normal expected load
    { duration: '30s', target: 100 }, // Ramp up to 100 users
    // { duration: '5m', target: 100 }, // Stay at 100 users
    // { duration: '2m', target: 200 }, // Ramp up to 200 users
    // { duration: '5m', target: 200 }, // Stay at 200 users
    // { duration: '2m', target: 0 },   // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms
    http_req_failed: ['rate<0.05'],   // Error rate must be less than 5%
    errors: ['rate<0.05'],            // Custom error rate less than 5%
  },
};

const BASE_URL = 'http://127.0.0.1:3000';

// Test data
const testUsers = [
  { name: 'Alice', email: 'alice@example.com' },
  { name: 'Bob', email: 'bob@example.com' },
  { name: 'Charlie', email: 'charlie@example.com' },
];

const testMessages = [
  { message: 'Hello World', priority: 'high' },
  { message: 'Test Message', priority: 'medium' },
  { message: 'Performance Test', priority: 'low' },
];

export function setup() {
  // Verify server is running
  const response = http.get(`${BASE_URL}/`);
  if (response.status !== 200) {
    throw new Error('Server is not responding correctly');
  }
  console.log('✅ Server is ready for testing');
  return { serverReady: true };
}

export default function () {
  const testScenario = Math.random();

  if (testScenario < 0.3) {
    // 30% - Test basic health check
    testHealthCheck();
  } else if (testScenario < 0.6) {
    // 30% - Test user operations
    testUserOperations();
  } else if (testScenario < 0.8) {
    // 20% - Test MQTT operations
    testMqttOperations();
  } else {
    // 20% - Test channel operations
    testChannelOperations();
  }

  sleep(1); // Wait 1 second between iterations
}

function testHealthCheck() {
  const response = http.get(`${BASE_URL}/`);

  const success = check(response, {
    'health check status is 200': (r) => r.status === 200,
    'health check response time OK': (r) => r.timings.duration < 100,
    'health check body contains Hello': (r) => r.body && r.body.includes('Hello'),
  });

  if (!success) {
    errorRate.add(1);
  }
}

function testUserOperations() {
  // Test getting all users
  let response = http.get(`${BASE_URL}/user/users`);
  let success = check(response, {
    'get users status is 200': (r) => r.status === 200,
    'get users response time OK': (r) => r.timings.duration < 200,
  });

  if (!success) {
    errorRate.add(1);
    return;
  }

  // Test creating a user
  const userData = testUsers[Math.floor(Math.random() * testUsers.length)];
  response = http.post(`${BASE_URL}/user/users`, JSON.stringify(userData), {
    headers: { 'Content-Type': 'application/json' },
  });

  success = check(response, {
    'create user status is 200 or 201': (r) => r.status === 200 || r.status === 201,
    'create user response time OK': (r) => r.timings.duration < 300,
  });

  if (!success) {
    errorRate.add(1);
    return;
  }

  // Test getting a specific user (if we have an ID)
  response = http.get(`${BASE_URL}/user/users/1`);
  check(response, {
    'get specific user response time OK': (r) => r.timings.duration < 200,
  });
}

function testMqttOperations() {
  // Test MQTT publisher
  const messageData = testMessages[Math.floor(Math.random() * testMessages.length)];
  let response = http.post(`${BASE_URL}/mqtt/pub`, JSON.stringify(messageData), {
    headers: { 'Content-Type': 'application/json' },
  });

  let success = check(response, {
    'mqtt publish status is 200': (r) => r.status === 200,
    'mqtt publish response time OK': (r) => r.timings.duration < 500,
  });

  if (!success) {
    errorRate.add(1);
    return;
  }

  // Test MQTT consumer
  response = http.get(`${BASE_URL}/mqtt/consume`);
  success = check(response, {
    'mqtt consume response time OK': (r) => r.timings.duration < 300,
  });

  if (!success) {
    errorRate.add(1);
  }
}

function testChannelOperations() {
  // Test channel publication
  const channelData = {
    user_id: Math.floor(Math.random() * 100) + 1,
    message: 'Channel test message',
    timestamp: new Date().toISOString(),
  };

  const response = http.post(`${BASE_URL}/channel/pub`, JSON.stringify(channelData), {
    headers: { 'Content-Type': 'application/json' },
  });

  const success = check(response, {
    'channel publish response time OK': (r) => r.timings.duration < 400,
  });

  if (!success) {
    errorRate.add(1);
  }
}

export function teardown(data) {
  console.log('🏁 Performance test completed');
  console.log('📊 Check the summary for detailed metrics');
} 