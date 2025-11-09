#!/bin/bash
set -euo pipefail

cd /root/qposit

# Create database file if it doesn't exist
touch database.sqlite3

# Run migrations, uses .env for database url
sqlx migrate run

cargo build --release

# Reload systemd in case service file changed
cp /root/qposit/qposit.service /etc/systemd/system/qposit.service
systemctl daemon-reload

cp /root/qposit/Caddyfile /etc/caddy/Caddyfile

# Restart services
systemctl restart qposit

# Reload Caddy config without downtime (or start if not running)
systemctl reload-or-restart caddy

# Check status
if systemctl is-active --quiet qposit; then
    echo "✓ qposit running"
else
    echo "✗ qposit failed to start"
    journalctl -u qposit -n 20 --no-pager
    exit 1
fi

if systemctl is-active --quiet caddy; then
    echo "✓ Caddy running"
else
    echo "✗ Caddy failed to start"
    journalctl -u caddy -n 20 --no-pager
    exit 1
fi

