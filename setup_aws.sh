#!/bin/bash
# =============================================================================
# VolunteerMatch Service — Deploy to AWS EC2
# Usage: EC2_IP=<public-ip> bash setup_aws.sh
# Requires: labsuser.pem in ~/Study/CS366/ (already downloaded from Lab)
# =============================================================================
set -euo pipefail

EC2_IP="${EC2_IP:-}"
PEM_KEY="${PEM_KEY:-$HOME/Study/CS366/labsuser.pem}"
EC2_USER="ec2-user"
APP_DIR="/opt/volunteer-match"
SERVICE_NAME="volunteer-match"
BINARY="volunteer_match_service"
LOCAL_SRC="$(cd "$(dirname "$0")" && pwd)"

# ── Validate inputs ───────────────────────────────────────────────────────────
if [[ -z "$EC2_IP" ]]; then
    echo "ERROR: EC2_IP is required"
    echo "Usage: EC2_IP=<public-ip> bash setup_aws.sh"
    exit 1
fi

if [[ ! -f "$PEM_KEY" ]]; then
    echo "ERROR: PEM key not found at $PEM_KEY"
    echo "Download labsuser.pem from the Lab's AWS Details panel"
    exit 1
fi

chmod 400 "$PEM_KEY"

SSH="ssh -i $PEM_KEY -o StrictHostKeyChecking=no $EC2_USER@$EC2_IP"
SCP="scp -i $PEM_KEY -o StrictHostKeyChecking=no"

echo "=== Deploying VolunteerMatch Service ==="
echo "EC2:  $EC2_USER@$EC2_IP"
echo "Key:  $PEM_KEY"
echo ""

# ── Step 1: Build Linux binary (cross-compile from Mac) ──────────────────────
echo ">>> [1/5] Building Linux x86_64 binary (cross-compile)..."
cd "$LOCAL_SRC"

# Install tools if missing
if ! command -v zig &>/dev/null; then
    echo "    Installing zig via brew..."
    brew install zig
fi
if ! cargo zigbuild --version &>/dev/null 2>&1; then
    echo "    Installing cargo-zigbuild..."
    cargo install cargo-zigbuild
fi
rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true

cargo zigbuild --release --target x86_64-unknown-linux-gnu
echo "    Built: target/x86_64-unknown-linux-gnu/release/$BINARY"

# ── Step 2: Upload files to EC2 ───────────────────────────────────────────────
echo ">>> [2/5] Uploading files to EC2..."
$SSH "mkdir -p ~/volunteer-match-service/migrations"
$SCP "target/x86_64-unknown-linux-gnu/release/$BINARY" "$EC2_USER@$EC2_IP:~/volunteer-match-service/"
$SCP -r "migrations/"                "$EC2_USER@$EC2_IP:~/volunteer-match-service/"

# Upload .env if it exists
if [[ -f "$LOCAL_SRC/.env" ]]; then
    $SCP "$LOCAL_SRC/.env" "$EC2_USER@$EC2_IP:~/volunteer-match-service/.env"
    echo "    Uploaded .env"
else
    echo "    WARNING: .env not found — create it on EC2 before starting the service"
    echo "    Template: $LOCAL_SRC/.env.example"
fi

# ── Step 3: Install on EC2 ────────────────────────────────────────────────────
echo ">>> [3/5] Installing on EC2..."
$SSH "sudo mkdir -p $APP_DIR && \
      sudo cp ~/volunteer-match-service/$BINARY $APP_DIR/ && \
      sudo cp -r ~/volunteer-match-service/migrations $APP_DIR/ && \
      sudo cp ~/volunteer-match-service/.env $APP_DIR/ 2>/dev/null || true && \
      sudo chmod +x $APP_DIR/$BINARY"

# ── Step 4: Create systemd service ───────────────────────────────────────────
echo ">>> [4/5] Setting up systemd service..."
$SSH "sudo tee /etc/systemd/system/${SERVICE_NAME}.service > /dev/null <<'UNIT'
[Unit]
Description=VolunteerMatch Service
After=network.target

[Service]
Type=simple
User=ec2-user
WorkingDirectory=${APP_DIR}
EnvironmentFile=${APP_DIR}/.env
ExecStart=${APP_DIR}/${BINARY}
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
UNIT"

$SSH "sudo systemctl daemon-reload && \
      sudo systemctl enable $SERVICE_NAME && \
      sudo systemctl restart $SERVICE_NAME"

# ── Step 5: Verify ────────────────────────────────────────────────────────────
echo ">>> [5/5] Verifying..."
sleep 3
$SSH "sudo systemctl is-active $SERVICE_NAME && echo 'Service is running'" || \
    echo "WARNING: Service may not be running — check logs below"

echo ""
echo "=== Deploy Complete ==="
echo "Health check: curl http://$EC2_IP:8080/health"
echo "Logs:         ssh -i $PEM_KEY $EC2_USER@$EC2_IP 'sudo journalctl -u $SERVICE_NAME -f'"
echo ""
echo "Run demo:     EC2_IP=$EC2_IP bash demo.sh"
