# ChessMate Load Testing Guide

This guide explains how to perform load testing on the ChessMate multiplayer server to ensure it can handle concurrent connections and game sessions.

## Prerequisites

Install Artillery (Node.js-based load testing tool):

```bash
npm install -g artillery
```

Verify installation:

```bash
artillery --version
```

## Running Load Tests

### 1. Start the Server

First, start the ChessMate server locally:

```bash
./scripts/run_server.sh
```

Or using Docker:

```bash
docker-compose up server
```

### 2. Run Basic Load Test

Execute the default load test configuration:

```bash
artillery run tests/load_test.yml
```

This will run a 2.5-minute test with multiple phases:
- **Warm-up** (30s): 2 users/second
- **Sustained load** (60s): 10 users/second
- **Traffic spike** (30s): 25 users/second
- **Cool-down** (30s): 5 users/second

### 3. Generate HTML Report

For a more detailed visual report:

```bash
artillery run --output report.json tests/load_test.yml
artillery report report.json --output report.html
open report.html  # macOS
xdg-open report.html  # Linux
start report.html  # Windows
```

### 4. Custom Load Tests

#### Quick Smoke Test

Test with minimal load (good for CI/CD):

```bash
artillery quick --count 10 --num 50 ws://localhost:3000/ws
```

This creates 10 virtual users making 50 requests each.

#### Sustained Load Test

Test server stability under constant load:

```bash
artillery run tests/load_test.yml \
  --overrides '{"config":{"phases":[{"duration":300,"arrivalRate":15}]}}'
```

This runs a 5-minute test with 15 users/second.

#### Spike Test

Test how server handles sudden traffic increases:

```bash
artillery run tests/load_test.yml \
  --overrides '{"config":{"phases":[{"duration":10,"arrivalRate":1},{"duration":20,"arrivalRate":100},{"duration":10,"arrivalRate":1}]}}'
```

This creates a traffic spike of 100 users/second for 20 seconds.

#### Stress Test

Find the server's breaking point:

```bash
artillery run tests/load_test.yml \
  --overrides '{"config":{"phases":[{"duration":60,"arrivalRate":50,"rampTo":200}]}}'
```

This gradually increases load from 50 to 200 users/second over 60 seconds.

## Understanding Results

Artillery provides several key metrics:

### Response Time Metrics

- **p50 (median)**: 50% of requests completed faster than this
- **p95**: 95% of requests completed faster than this
- **p99**: 99% of requests completed faster than this

**Target values:**
- p50 < 100ms (excellent)
- p95 < 200ms (good)
- p99 < 500ms (acceptable)

### Throughput Metrics

- **Requests per second (RPS)**: Number of requests handled per second
- **Virtual users (VUs)**: Number of concurrent simulated users

### Error Metrics

- **Error rate**: Percentage of failed requests
  - Target: < 1% error rate
  - Acceptable: < 5% error rate
  - Concerning: > 10% error rate

- **Timeout rate**: Percentage of requests that timed out
  - Target: 0%

### Example Output

```
Summary report @ 14:23:45(-0700)
──────────────────────────────────────────────────────────────
  Scenarios launched:  1500
  Scenarios completed: 1485
  Requests completed:  4455
  Mean response/sec:   29.7
  Response time (msec):
    min: 12
    max: 456
    median: 45
    p95: 178
    p99: 312
  Scenario counts:
    Matchmaking and gameplay: 1050 (70%)
    Connect and disconnect: 450 (30%)
  Codes:
    0: 4455
  Errors:
    WebSocket connection failed: 15
```

### Interpreting Results

**Good Performance Indicators:**
- ✅ Error rate < 1%
- ✅ p99 response time < 500ms
- ✅ Scenarios completed ≈ Scenarios launched
- ✅ No timeout errors
- ✅ CPU usage < 80%
- ✅ Memory usage stable

**Warning Signs:**
- ⚠️ Error rate 1-5%
- ⚠️ p99 response time 500-1000ms
- ⚠️ Increasing response times over test duration
- ⚠️ CPU usage > 80%
- ⚠️ Memory usage climbing

**Critical Issues:**
- ❌ Error rate > 5%
- ❌ p99 response time > 1000ms
- ❌ Many timeout errors
- ❌ Scenarios completed << Scenarios launched
- ❌ CPU usage at 100%
- ❌ Memory leaks (constantly increasing memory)

## Load Test Scenarios

The default configuration includes three scenarios:

### 1. Matchmaking and Gameplay (50%)

Simulates full game flow:
1. Connect to WebSocket
2. Join matchmaking queue
3. Wait for opponent
4. Make several moves
5. Disconnect

This tests the complete user journey.

### 2. Connect and Disconnect (30%)

Stress tests connection handling:
1. Connect
2. Join matchmaking
3. Hold connection briefly
4. Disconnect

This tests connection pool management.

### 3. Reconnection Test (20%)

Tests rapid reconnection handling:
1. Connect
2. Join
3. Disconnect quickly
4. Repeat 5 times

This tests cleanup and resource management.

## Performance Targets

### Local Development (1 core, 2GB RAM)

- **Concurrent games**: 50-100
- **WebSocket connections**: 200+
- **p99 latency**: < 100ms

### Production (2 cores, 4GB RAM)

- **Concurrent games**: 500-1000
- **WebSocket connections**: 2000+
- **p99 latency**: < 200ms

### Scaled Production (4+ cores, 8GB+ RAM)

- **Concurrent games**: 2000+
- **WebSocket connections**: 5000+
- **p99 latency**: < 300ms

## Monitoring During Load Tests

### Server Metrics

Monitor server resources while running load tests:

```bash
# Watch server logs
docker logs -f chessmate-server

# Monitor resource usage
docker stats chessmate-server

# Check active connections
netstat -an | grep :3000 | grep ESTABLISHED | wc -l
```

### Application Metrics

Add custom logging in the server to track:
- Active games count
- Matchmaking queue size
- Move processing time
- Error rates by type

## Troubleshooting

### High Error Rates

**Problem**: Error rate > 5%

**Possible causes:**
- Server overloaded (check CPU/memory)
- Database connection pool exhausted
- Too many open files (check `ulimit -n`)
- Network issues

**Solutions:**
- Increase server resources
- Optimize database queries
- Increase file descriptor limit: `ulimit -n 10000`
- Add rate limiting

### Slow Response Times

**Problem**: p99 > 1000ms

**Possible causes:**
- Slow database queries
- Inefficient game logic
- Lock contention
- Network latency

**Solutions:**
- Profile code to find bottlenecks
- Add database indexes
- Reduce lock scope
- Use connection pooling

### Memory Leaks

**Problem**: Memory usage continuously increases

**Possible causes:**
- Games not being cleaned up
- WebSocket connections not closed properly
- Event listeners not removed

**Solutions:**
- Audit game cleanup logic
- Ensure proper WebSocket close handling
- Profile with memory profiler

### Connection Failures

**Problem**: "WebSocket connection failed" errors

**Possible causes:**
- Server at max connections
- Firewall blocking connections
- Server crashed

**Solutions:**
- Increase max connections limit
- Check firewall rules
- Review server logs for crashes
- Add health checks

## Continuous Integration

Add load testing to CI/CD pipeline:

```yaml
# .github/workflows/load-test.yml
name: Load Test

on: [pull_request]

jobs:
  load-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Start server
        run: docker-compose up -d server

      - name: Wait for server
        run: sleep 10

      - name: Install Artillery
        run: npm install -g artillery

      - name: Run load test
        run: artillery run tests/load_test.yml

      - name: Check results
        run: |
          if [ $? -ne 0 ]; then
            echo "Load test failed!"
            exit 1
          fi
```

## Best Practices

1. **Start Small**: Begin with low load and gradually increase
2. **Monitor Resources**: Watch CPU, memory, and network during tests
3. **Test Realistic Scenarios**: Match actual user behavior patterns
4. **Run Multiple Times**: Results can vary, average multiple runs
5. **Test in Production-Like Environment**: Use similar hardware/network
6. **Set Baselines**: Record performance metrics to track over time
7. **Test After Changes**: Run load tests before deploying changes
8. **Document Results**: Keep a log of load test results and server configurations

## Advanced Configuration

### Custom Environment Variables

Test against different environments:

```bash
# Test staging server
artillery run tests/load_test.yml \
  --environment staging \
  --target wss://staging.chessmate.com

# Test with authentication
artillery run tests/load_test.yml \
  --variables '{"authToken":"your-token-here"}'
```

### Distributed Load Testing

For higher load, run Artillery from multiple machines:

```bash
# Machine 1
artillery run tests/load_test.yml --output report1.json

# Machine 2
artillery run tests/load_test.yml --output report2.json

# Combine reports
artillery report report1.json report2.json --output combined.html
```

## Resources

- [Artillery Documentation](https://www.artillery.io/docs)
- [WebSocket Load Testing Guide](https://www.artillery.io/docs/guides/guides/websocket-testing)
- [Performance Testing Best Practices](https://www.artillery.io/docs/guides/getting-started/writing-your-first-test)

## Next Steps

After load testing:
1. Identify bottlenecks
2. Optimize server code
3. Scale infrastructure if needed
4. Set up continuous performance monitoring
5. Create alerts for performance degradation
