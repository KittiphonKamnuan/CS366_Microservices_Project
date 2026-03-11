#!/bin/bash
# =============================================================================
# AWS Resource Setup: RDS PostgreSQL + SNS Topics
# Run from your local machine (with AWS CLI configured) or AWS CloudShell
# =============================================================================
set -euo pipefail

REGION="us-east-1"
DB_INSTANCE_ID="volunteer-db"
DB_NAME="volunteer_db"
DB_USER="volunteer_user"
DB_PASS="ChangeMe123!"   # Change this
DB_CLASS="db.t3.micro"
SNS_LOCATION="volunteer-location-updated"
SNS_MATCH="match-status-changed"

# ── RDS PostgreSQL ────────────────────────────────────────────────────────────
echo ">>> Creating RDS PostgreSQL instance..."
aws rds create-db-instance \
    --region "$REGION" \
    --db-instance-identifier "$DB_INSTANCE_ID" \
    --db-instance-class "$DB_CLASS" \
    --engine postgres \
    --engine-version "15" \
    --master-username "$DB_USER" \
    --master-user-password "$DB_PASS" \
    --db-name "$DB_NAME" \
    --allocated-storage 20 \
    --storage-type gp2 \
    --no-multi-az \
    --no-publicly-accessible \
    --backup-retention-period 0 \
    --no-deletion-protection \
    --no-enable-performance-insights

echo ">>> Waiting for RDS instance to be available (this takes ~5 min)..."
aws rds wait db-instance-available \
    --region "$REGION" \
    --db-instance-identifier "$DB_INSTANCE_ID"

RDS_ENDPOINT=$(aws rds describe-db-instances \
    --region "$REGION" \
    --db-instance-identifier "$DB_INSTANCE_ID" \
    --query "DBInstances[0].Endpoint.Address" \
    --output text)

echo ">>> RDS endpoint: $RDS_ENDPOINT"
echo "DATABASE_URL=postgresql://${DB_USER}:${DB_PASS}@${RDS_ENDPOINT}:5432/${DB_NAME}"

# ── SNS Topics ────────────────────────────────────────────────────────────────
echo ">>> Creating SNS topics..."
LOCATION_ARN=$(aws sns create-topic \
    --region "$REGION" \
    --name "$SNS_LOCATION" \
    --query TopicArn --output text)

MATCH_ARN=$(aws sns create-topic \
    --region "$REGION" \
    --name "$SNS_MATCH" \
    --query TopicArn --output text)

echo ">>> SNS_LOCATION_TOPIC_ARN=$LOCATION_ARN"
echo ">>> SNS_MATCH_TOPIC_ARN=$MATCH_ARN"

# ── Print .env ────────────────────────────────────────────────────────────────
echo ""
echo "=== Add to your .env file ==="
cat <<EOF
DATABASE_URL=postgresql://${DB_USER}:${DB_PASS}@${RDS_ENDPOINT}:5432/${DB_NAME}
SNS_LOCATION_TOPIC_ARN=${LOCATION_ARN}
SNS_MATCH_TOPIC_ARN=${MATCH_ARN}
HOST=0.0.0.0
PORT=8080
RUST_LOG=info
EOF
