#!/bin/bash
# =============================================================================
# Demo script — VolunteerMatch Service
# Usage: BASE_URL=http://<ec2-ip>:8080 ./demo.sh
# =============================================================================
BASE_URL="${BASE_URL:-http://localhost:8080}"

echo "=== VolunteerMatch Service Demo ==="
echo "Base URL: $BASE_URL"
echo ""

# ── Health check ──────────────────────────────────────────────────────────────
echo "--- Health Check ---"
curl -s "$BASE_URL/health" | python3 -m json.tool
echo ""

# ── 1. Register volunteer (SYNC) ──────────────────────────────────────────────
echo "--- [SYNC] Register Volunteer ---"
VOL_RESP=$(curl -s -X POST "$BASE_URL/volunteers" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "สมชาย ใจดี",
    "phone": "0812345678",
    "skills": ["driving", "boat"],
    "area": "LOC-001"
  }')
echo "$VOL_RESP" | python3 -m json.tool
VOL_ID=$(echo "$VOL_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['volunteer_id'])")
echo "Volunteer ID: $VOL_ID"
echo ""

# ── 2. Create task ────────────────────────────────────────────────────────────
echo "--- [SYNC] Create Task ---"
TASK_RESP=$(curl -s -X POST "$BASE_URL/tasks" \
  -H "Content-Type: application/json" \
  -d '{
    "incident_id": "INC-001",
    "title": "ขนกระสอบทราย",
    "required_skills": ["driving", "boat"],
    "location_id": "LOC-001",
    "volunteers_needed": 3,
    "urgency": "high"
  }')
echo "$TASK_RESP" | python3 -m json.tool
TASK_ID=$(echo "$TASK_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['task_id'])")
echo "Task ID: $TASK_ID"
echo ""

# ── 3. Search tasks (SYNC) ────────────────────────────────────────────────────
echo "--- [SYNC] Search Tasks (location=LOC-001, skill=driving) ---"
curl -s "$BASE_URL/tasks?location_id=LOC-001&required_skills=driving" | python3 -m json.tool
echo ""

# ── 4. Match volunteer to task (SYNC) ─────────────────────────────────────────
echo "--- [SYNC] Match Volunteer to Task ---"
MATCH_RESP=$(curl -s -X POST "$BASE_URL/tasks/$TASK_ID/match" \
  -H "Content-Type: application/json" \
  -d "{\"volunteer_id\": \"$VOL_ID\"}")
echo "$MATCH_RESP" | python3 -m json.tool
echo ""

# ── 5. Idempotency check ──────────────────────────────────────────────────────
echo "--- [SYNC] Idempotency: Match same volunteer again (should return 200 + note) ---"
curl -s -X POST "$BASE_URL/tasks/$TASK_ID/match" \
  -H "Content-Type: application/json" \
  -d "{\"volunteer_id\": \"$VOL_ID\"}" | python3 -m json.tool
echo ""

# ── 6. Update GPS location (SYNC + triggers ASYNC SNS event) ─────────────────
echo "--- [SYNC+ASYNC] Update Volunteer GPS Location ---"
echo "(This also publishes volunteer.location_updated to SNS asynchronously)"
curl -s -X PATCH "$BASE_URL/volunteers/$VOL_ID/location" \
  -H "Content-Type: application/json" \
  -d '{"lat": 13.7563, "lng": 100.5018}' | python3 -m json.tool
echo ""

# ── 7. Get GPS location (SYNC) ────────────────────────────────────────────────
echo "--- [SYNC] Get Volunteer GPS Location ---"
curl -s "$BASE_URL/volunteers/$VOL_ID/location" | python3 -m json.tool
echo ""

# ── 8. Validation demo ────────────────────────────────────────────────────────
echo "--- [ERROR] Invalid GPS (out of Thailand bounds) ---"
curl -s -X PATCH "$BASE_URL/volunteers/$VOL_ID/location" \
  -H "Content-Type: application/json" \
  -d '{"lat": 99.0, "lng": 200.0}' | python3 -m json.tool
echo ""

echo "=== Demo Complete ==="
