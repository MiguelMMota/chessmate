# ChessMate Server Deployment Guide

This guide covers deploying the ChessMate multiplayer server to production environments.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Configuration](#environment-configuration)
3. [Docker Deployment](#docker-deployment)
4. [Cloud Platform Deployment](#cloud-platform-deployment)
5. [Database Setup](#database-setup)
6. [Load Balancing & Scaling](#load-balancing--scaling)
7. [Monitoring & Logging](#monitoring--logging)
8. [Security Best Practices](#security-best-practices)
9. [Maintenance & Updates](#maintenance--updates)

## Prerequisites

### Required Tools

- Docker (20.10+)
- Docker Compose (2.0+)
- PostgreSQL (15+)
- SSL/TLS certificates (for production)

### Cloud Provider Account (choose one)

- AWS (EC2, RDS, ELB)
- Google Cloud Platform (Compute Engine, Cloud SQL, Load Balancer)
- DigitalOcean (Droplets, Managed Databases, Load Balancer)
- Any VPS provider with Docker support

## Environment Configuration

### Required Environment Variables

Create a `.env` file with the following variables:

```bash
# Database
DATABASE_URL=postgres://username:password@host:5432/chessmate

# Server
RUST_LOG=info
PORT=3000

# Security (future)
JWT_SECRET=your-secret-key-here
ALLOWED_ORIGINS=https://yourdomain.com

# Performance
MAX_CONNECTIONS=1000
WORKER_THREADS=4
```

### Production .env Example

```bash
DATABASE_URL=postgres://chessmate_user:secure_password@db.example.com:5432/chessmate_prod
RUST_LOG=warn
PORT=3000
MAX_CONNECTIONS=5000
WORKER_THREADS=8
```

## Docker Deployment

### Single-Node Docker Deployment

**1. Build the production image:**

```bash
docker build -t chessmate-server:latest .
```

**2. Run with Docker Compose:**

```bash
docker-compose -f docker-compose.prod.yml up -d
```

**3. Verify deployment:**

```bash
docker ps
docker logs chessmate-server
```

### Production Docker Compose

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  db:
    image: postgres:15-alpine
    container_name: chessmate-db-prod
    environment:
      POSTGRES_USER: ${DB_USER}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: chessmate_prod
    volumes:
      - postgres_data_prod:/var/lib/postgresql/data
    restart: always
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

  server:
    image: chessmate-server:latest
    container_name: chessmate-server-prod
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://${DB_USER}:${DB_PASSWORD}@db:5432/chessmate_prod
      RUST_LOG: warn
    depends_on:
      db:
        condition: service_healthy
    restart: always
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G

volumes:
  postgres_data_prod:
    driver: local
```

## Cloud Platform Deployment

### AWS Deployment

#### Option 1: EC2 + RDS

**1. Launch EC2 instance:**

```bash
# Use Amazon Linux 2 AMI
aws ec2 run-instances \
  --image-id ami-xxxxx \
  --instance-type t3.medium \
  --key-name your-key \
  --security-group-ids sg-xxxxx
```

**2. Set up RDS PostgreSQL:**

```bash
aws rds create-db-instance \
  --db-instance-identifier chessmate-db \
  --db-instance-class db.t3.micro \
  --engine postgres \
  --master-username admin \
  --master-user-password yourpassword \
  --allocated-storage 20
```

**3. SSH into EC2 and deploy:**

```bash
ssh -i your-key.pem ec2-user@your-instance-ip

# Install Docker
sudo yum update -y
sudo yum install docker -y
sudo service docker start
sudo usermod -a -G docker ec2-user

# Clone and deploy
git clone <your-repo>
cd chessmate
docker-compose -f docker-compose.prod.yml up -d
```

**4. Configure security groups:**

- Port 3000 (WebSocket) - inbound from load balancer
- Port 5432 (PostgreSQL) - inbound from EC2 security group only

#### Option 2: ECS (Elastic Container Service)

**1. Push image to ECR:**

```bash
aws ecr create-repository --repository-name chessmate-server
docker tag chessmate-server:latest <account-id>.dkr.ecr.us-east-1.amazonaws.com/chessmate-server
docker push <account-id>.dkr.ecr.us-east-1.amazonaws.com/chessmate-server
```

**2. Create ECS task definition:**

```json
{
  "family": "chessmate-server",
  "networkMode": "awsvpc",
  "containerDefinitions": [
    {
      "name": "chessmate-server",
      "image": "<account-id>.dkr.ecr.us-east-1.amazonaws.com/chessmate-server",
      "portMappings": [
        {
          "containerPort": 3000,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "DATABASE_URL",
          "value": "postgres://user:pass@rds-endpoint:5432/chessmate"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/chessmate",
          "awslogs-region": "us-east-1"
        }
      }
    }
  ],
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024"
}
```

**3. Create ECS service with load balancer:**

```bash
aws ecs create-service \
  --cluster chessmate-cluster \
  --service-name chessmate-server \
  --task-definition chessmate-server \
  --desired-count 2 \
  --launch-type FARGATE \
  --load-balancers targetGroupArn=<tg-arn>,containerName=chessmate-server,containerPort=3000
```

### Google Cloud Platform Deployment

**1. Create Compute Engine instance:**

```bash
gcloud compute instances create chessmate-server \
  --machine-type=e2-medium \
  --image-family=ubuntu-2004-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=20GB
```

**2. Set up Cloud SQL (PostgreSQL):**

```bash
gcloud sql instances create chessmate-db \
  --database-version=POSTGRES_15 \
  --tier=db-f1-micro \
  --region=us-central1
```

**3. Deploy with Docker:**

```bash
# SSH into instance
gcloud compute ssh chessmate-server

# Install Docker and deploy (similar to AWS)
```

### DigitalOcean Deployment

**1. Create Droplet:**

```bash
doctl compute droplet create chessmate-server \
  --image docker-20-04 \
  --size s-2vcpu-2gb \
  --region nyc3
```

**2. Create Managed Database:**

```bash
doctl databases create chessmate-db \
  --engine postgres \
  --region nyc3 \
  --size db-s-1vcpu-1gb
```

**3. Deploy application:**

```bash
# SSH into Droplet
ssh root@your-droplet-ip

# Clone and run
git clone <your-repo>
cd chessmate
docker-compose -f docker-compose.prod.yml up -d
```

## Database Setup

### Initial Database Configuration

**1. Create production database:**

```sql
CREATE DATABASE chessmate_prod;
CREATE USER chessmate_user WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE chessmate_prod TO chessmate_user;
```

**2. Run migrations (future):**

```bash
# Using sqlx-cli
sqlx migrate run --database-url $DATABASE_URL
```

### Database Backup Strategy

**Automated daily backups:**

```bash
#!/bin/bash
# backup.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="chessmate_backup_${DATE}.sql"

pg_dump -h db.example.com -U chessmate_user -d chessmate_prod > $BACKUP_FILE
gzip $BACKUP_FILE

# Upload to S3 or Cloud Storage
aws s3 cp ${BACKUP_FILE}.gz s3://your-backup-bucket/
```

**Schedule with cron:**

```bash
# Run daily at 2 AM
0 2 * * * /path/to/backup.sh
```

### Database Performance Tuning

**PostgreSQL configuration (`postgresql.conf`):**

```ini
max_connections = 100
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 2621kB
min_wal_size = 1GB
max_wal_size = 4GB
```

## Load Balancing & Scaling

### Single Server Architecture

```
Internet → [Load Balancer] → [Server Instance] → [Database]
```

### Horizontal Scaling with Sticky Sessions

**NGINX Load Balancer configuration:**

```nginx
upstream chessmate_servers {
    ip_hash;  # Sticky sessions
    server 10.0.1.10:3000;
    server 10.0.1.11:3000;
    server 10.0.1.12:3000;
}

server {
    listen 443 ssl http2;
    server_name api.chessmate.com;

    ssl_certificate /etc/ssl/certs/chessmate.crt;
    ssl_certificate_key /etc/ssl/private/chessmate.key;

    location /ws {
        proxy_pass http://chessmate_servers;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

        # WebSocket timeouts
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }

    location /health {
        proxy_pass http://chessmate_servers;
    }
}
```

### Auto-Scaling Configuration (AWS)

**Auto Scaling Group:**

```bash
aws autoscaling create-auto-scaling-group \
  --auto-scaling-group-name chessmate-asg \
  --launch-template LaunchTemplateId=lt-xxxxx \
  --min-size 2 \
  --max-size 10 \
  --desired-capacity 2 \
  --target-group-arns <tg-arn>
```

**Scaling Policies:**

```bash
# Scale up when CPU > 70%
aws autoscaling put-scaling-policy \
  --auto-scaling-group-name chessmate-asg \
  --policy-name scale-up \
  --scaling-adjustment 1 \
  --adjustment-type ChangeInCapacity

# Scale down when CPU < 30%
aws autoscaling put-scaling-policy \
  --auto-scaling-group-name chessmate-asg \
  --policy-name scale-down \
  --scaling-adjustment -1 \
  --adjustment-type ChangeInCapacity
```

## Monitoring & Logging

### Health Checks

**Add health check endpoint to server:**

```rust
// In src/bin/server.rs
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Add to router
.route("/health", get(health_check))
```

### Logging with CloudWatch/Stackdriver

**Configure logging:**

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

### Metrics Collection

**Prometheus metrics (future):**

```rust
use prometheus::{register_counter, register_histogram, Encoder};

lazy_static! {
    static ref GAMES_CREATED: Counter = register_counter!(
        "chessmate_games_created_total",
        "Total number of games created"
    ).unwrap();

    static ref MOVE_LATENCY: Histogram = register_histogram!(
        "chessmate_move_latency_seconds",
        "Time to process a move"
    ).unwrap();
}
```

### Alerting

**CloudWatch Alarms (AWS):**

```bash
aws cloudwatch put-metric-alarm \
  --alarm-name high-cpu \
  --alarm-description "CPU usage > 80%" \
  --metric-name CPUUtilization \
  --namespace AWS/EC2 \
  --statistic Average \
  --period 300 \
  --threshold 80 \
  --comparison-operator GreaterThanThreshold \
  --evaluation-periods 2
```

## Security Best Practices

### 1. TLS/SSL Configuration

**Obtain SSL certificate (Let's Encrypt):**

```bash
sudo certbot --nginx -d api.chessmate.com
```

**Force HTTPS:**

```nginx
server {
    listen 80;
    server_name api.chessmate.com;
    return 301 https://$server_name$request_uri;
}
```

### 2. Firewall Configuration

```bash
# UFW (Ubuntu)
sudo ufw allow 22/tcp     # SSH
sudo ufw allow 443/tcp    # HTTPS
sudo ufw allow 3000/tcp   # Application (from load balancer only)
sudo ufw enable
```

### 3. Database Security

- Use strong passwords (20+ characters, random)
- Enable SSL connections only
- Restrict database access to application servers only
- Regular security updates

### 4. Environment Variables

Never commit secrets to Git. Use:

- **AWS**: Secrets Manager or Parameter Store
- **GCP**: Secret Manager
- **Docker**: Docker secrets
- **Kubernetes**: Kubernetes secrets

Example with AWS Secrets Manager:

```bash
# Store secret
aws secretsmanager create-secret \
  --name chessmate/db-password \
  --secret-string "your-secure-password"

# Retrieve in application
aws secretsmanager get-secret-value --secret-id chessmate/db-password
```

### 5. Rate Limiting

**Add rate limiting middleware (future):**

```rust
use tower::limit::RateLimitLayer;
use std::time::Duration;

let app = Router::new()
    .route("/ws", get(websocket_handler))
    .layer(RateLimitLayer::new(100, Duration::from_secs(60))); // 100 req/min
```

## Maintenance & Updates

### Zero-Downtime Deployment

**Blue-Green Deployment:**

1. Deploy new version to "green" environment
2. Run health checks on green
3. Switch load balancer to green
4. Keep blue running for rollback
5. After validation, shut down blue

**Rolling Update (Kubernetes):**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chessmate-server
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 1
```

### Database Migration Strategy

```bash
# 1. Backup database
./backup.sh

# 2. Run migrations on staging
sqlx migrate run --database-url $STAGING_DB

# 3. Test on staging
./integration_tests.sh

# 4. Run migrations on production
sqlx migrate run --database-url $PROD_DB

# 5. Deploy new application version
docker-compose -f docker-compose.prod.yml up -d
```

### Monitoring Deployment

```bash
# Watch logs during deployment
docker logs -f chessmate-server-prod

# Check health endpoint
curl https://api.chessmate.com/health

# Monitor metrics
watch -n 5 'docker stats chessmate-server-prod'
```

## Troubleshooting

### Common Issues

**1. WebSocket connections failing:**

```bash
# Check firewall
sudo ufw status

# Check NGINX WebSocket config
sudo nginx -t
sudo systemctl restart nginx

# Check server logs
docker logs chessmate-server-prod
```

**2. Database connection errors:**

```bash
# Test database connectivity
psql -h db.example.com -U chessmate_user -d chessmate_prod

# Check connection limits
SELECT count(*) FROM pg_stat_activity;
```

**3. High memory usage:**

```bash
# Check Docker container stats
docker stats

# Restart container if needed
docker-compose restart server
```

## Performance Benchmarks

### Expected Performance (t3.medium EC2)

- **Concurrent games**: 500-1000
- **WebSocket connections**: 2000+
- **Move latency**: <50ms (p99)
- **Memory per game**: ~1MB
- **CPU per game**: <0.1%

### Load Testing

```bash
# Install websocket load testing tool
npm install -g artillery

# Create load test config (artillery.yml)
# Run load test
artillery run artillery.yml
```

## Cost Estimation

### AWS Monthly Costs (estimated)

- **EC2 t3.medium (2 instances)**: $60
- **RDS db.t3.micro**: $15
- **Application Load Balancer**: $20
- **Data transfer (100GB)**: $9
- **CloudWatch logs**: $5
- **Total**: ~$110/month

### DigitalOcean Monthly Costs (estimated)

- **Droplet (2GB RAM, 2 instances)**: $24
- **Managed Database**: $15
- **Load Balancer**: $12
- **Total**: ~$51/month

## Next Steps

1. Set up CI/CD pipeline for automated deployments
2. Configure monitoring and alerting
3. Implement database backup automation
4. Set up staging environment
5. Load test the deployment
6. Document incident response procedures

For development workflows and local testing, see [DEVELOPMENT.md](DEVELOPMENT.md).

For architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md).
