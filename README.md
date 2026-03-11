# VolunteerMatch Service

Microservice สำหรับจับคู่อาสาสมัครกับงานในระบบตอบสนองภัยพิบัติ

**Stack**: Rust, Actix-web 4, PostgreSQL, AWS SNS

---

## API Endpoints

| Method | Path | Type | Description |
|--------|------|------|-------------|
| GET | `/health` | Sync | Health check |
| POST | `/volunteers` | Sync | ลงทะเบียนอาสาสมัคร |
| PATCH | `/volunteers/{id}/location` | Sync + Async | อัปเดต GPS (publish SNS) |
| GET | `/volunteers/{id}/location` | Sync | ดู GPS ล่าสุด |
| POST | `/tasks` | Sync | สร้างงาน |
| GET | `/tasks` | Sync | ค้นหางาน |
| POST | `/tasks/{id}/match` | Sync + Async | จับคู่อาสากับงาน (publish SNS) |

---

## Run Local (Docker)

```bash
docker-compose up --build
curl http://localhost:8080/health
```

---

## Deploy บน AWS Cloud9

### 1. สร้าง Cloud9 Environment
AWS Console → Cloud9 → **Create environment**
- Instance type: `t3.small`
- Platform: Amazon Linux 2023
- Network: Secure Shell (SSH)

### 2. ติดตั้ง Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
sudo dnf install -y pkg-config openssl-devel
```

### 3. Clone โปรเจค

```bash
git clone https://github.com/KittiphonKamnuan/CS366_Microservices_Project.git
cd CS366_Microservices_Project
```

### 4. สร้าง RDS + SNS

```bash
bash setup_rds_sns.sh
```

### 5. สร้าง .env

```bash
cp .env.example .env
nano .env
# ใส่ DATABASE_URL และ SNS ARNs จาก step 4
```

### 6. Build และทดสอบ

```bash
cargo build --release
./target/release/volunteer_match_service
```

### 7. ติดตั้งเป็น systemd service

```bash
sudo cp target/release/volunteer_match_service /opt/
sudo cp -r migrations /opt/
sudo cp .env /opt/

sudo tee /etc/systemd/system/volunteer-match.service > /dev/null <<EOF
[Unit]
Description=VolunteerMatch Service
After=network.target

[Service]
Type=simple
User=ec2-user
WorkingDirectory=/opt
EnvironmentFile=/opt/.env
ExecStart=/opt/volunteer_match_service
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable volunteer-match
sudo systemctl start volunteer-match
```

### 8. เปิด Port 8080

EC2 Console → Security Groups ของ Cloud9 instance → Inbound rules → Add rule:
- Type: Custom TCP, Port: `8080`, Source: `0.0.0.0/0`

### 9. ทดสอบ

```bash
curl http://169.254.169.254/latest/meta-data/public-ipv4
BASE_URL=http://<public-ip>:8080 bash demo.sh
```

> **หมายเหตุ**: Cloud9 และ RDS ต้องอยู่ใน VPC เดียวกัน และ RDS Security Group ต้องเปิด port `5432` ให้ Cloud9 SG เข้าได้
