#!/bin/bash

# ===================================================================
# IPManager - Komplett-Setup (System, iPXE & GitHub Push)
# Stand: 19. Jan 2026
# ===================================================================

set -e

echo "--- Starte IPManager System-Setup für User: $USER ---"

# 1. System-Updates & Abhängigkeiten
sudo apt update && sudo apt upgrade -y
sudo apt install -y build-essential libssl-dev pkg-config postgresql postgresql-contrib dnsmasq curl git

# 2. Rust Installation
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# 3. Datenbank Setup
sudo -u postgres psql -c "CREATE USER ipmanager WITH PASSWORD 'admin123';" || true
sudo -u postgres psql -c "CREATE DATABASE ipmanager OWNER ipmanager;" || true

# 4. Verzeichnisstruktur & Berechtigungen
echo "Konfiguriere Verzeichnisse..."
sudo mkdir -p /var/lib/tftpboot/pxe-assets
sudo mkdir -p /etc/dnsmasq.d/

# Rechte für den aktuellen User setzen
sudo chown -R $USER:$USER /etc/dnsmasq.d/
sudo chown -R $USER:$USER /var/lib/tftpboot/

# 5. iPXE Images Download
echo "Lade iPXE Binaries herunter..."
IPXE_DIR="/var/lib/tftpboot/pxe-assets"
URLS=(
    "https://boot.ipxe.org/ipxe.lkrn"
    "https://boot.ipxe.org/ipxe.pxe"
    "https://boot.ipxe.org/ipxe.efi"
)

for url in "${URLS[@]}"; do
    filename=$(basename $url)
    if [ ! -f "$IPXE_DIR/$filename" ]; then
        echo "Downloade $filename..."
        curl -L -o "$IPXE_DIR/$filename" "$url"
    else
        echo "$filename existiert bereits, überspringe..."
    fi
done

# 6. Sudoers-Regel für dnsmasq Reload
SUDOERS_LINE="$USER ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart dnsmasq"
if ! sudo grep -q "$USER.*dnsmasq" /etc/sudoers; then
    echo "$SUDOERS_LINE" | sudo tee -a /etc/sudoers > /dev/null
fi

# 7. SQLx CLI
if ! command -v sqlx &> /dev/null; then
    cargo install sqlx-cli --no-default-features --features postgres
fi

# 8. GitHub Remote & Push Setup
echo "Konfiguriere Git und führe Push aus..."

# GitHub Hostkey zu known_hosts hinzufügen (behebt 'Host key verification failed')
mkdir -p ~/.ssh
ssh-keyscan github.com >> ~/.ssh/known_hosts 2>/dev/null

# Remote auf SSH umstellen
git remote set-url origin git@github.com:SyncLogic-2026/ipmanager.git || git remote add origin git@github.com:SyncLogic-2026/ipmanager.git

# Änderungen committen und pushen
git add .
# Prüfen, ob es Änderungen gibt, bevor wir committen
if ! git diff-index --quiet HEAD --; then
    git commit -m "Update: System Setup, iPXE images and configurations"
    git push -u origin main || git push -u origin master
else
    echo "Keine Änderungen zum Committen vorhanden."
fi

echo "--- Setup und Push erfolgreich abgeschlossen! ---"