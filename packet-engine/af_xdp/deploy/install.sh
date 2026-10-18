#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_NAME="aegis-probe"

install -d /etc/systemd/system/${SERVICE_NAME}.service.d
install -m 644 "${SCRIPT_DIR}/${SERVICE_NAME}.service" /etc/systemd/system/
install -m 644 "${SCRIPT_DIR}/${SERVICE_NAME}.socket" /etc/systemd/system/

systemctl daemon-reload
systemctl enable ${SERVICE_NAME}.socket ${SERVICE_NAME}.service

echo "Installed. Start with: systemctl start ${SERVICE_NAME}.service"
