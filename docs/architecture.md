# Architektur-Dokumentation: IPManager (SyncLogic-2026)

## 1. Übersicht

Der IPManager ist ein IP-Management-System (IPAM), das als "Source of Truth" für Netzwerkressourcen dient. Die Anwendung verwaltet Hosts, Subnetze und PXE-Boot-Konfigurationen in einer PostgreSQL-Datenbank und synchronisiert diese mit dem **dnsmasq** DHCP-Backend.

## 2. Kernkomponenten

### 2.1 Backend (Rust & Axum)

Das Herzstück der Anwendung ist in Rust geschrieben.

* **Web-Server:** Axum verarbeitet REST-Anfragen und rendert HTML-Templates (Tera).
* **Datenbank-Layer:** SQLx (v0.8) wird für asynchrone PostgreSQL-Abfragen genutzt.
* **DHCP-Modul:** Ein dediziertes Modul (`src/dhcp/dnsmasq.rs`) transformiert den Datenbank-State in dnsmasq-kompatible Konfigurationen.

### 2.2 Source of Truth (PostgreSQL)

Alle Daten (MAC-Adressen, IP-Reservierungen, Hostnames, PXE-Images) liegen in PostgreSQL. Es findet keine dauerhafte Speicherung von Zuständen im DHCP-Server selbst statt; dieser wird bei jeder Änderung neu provisioniert.

### 2.3 DHCP-Backend (dnsmasq)

Als DHCP-Server wird **dnsmasq** eingesetzt.

* **Konfiguration:** Erfolgt über eine dedizierte Datei unter `/etc/dnsmasq.d/01-rust-hosts.conf`.
* **Format:** `dhcp-host=MAC,IP,HOSTNAME`.
* **Synchronisation:** Das Rust-Backend schreibt die Datei atomar und sendet ein `SIGHUP`-Signal an den dnsmasq-Prozess, um die Änderungen ohne Dienstunterbrechung zu laden.

## 3. Datenfluss & Synchronisation

### 3.1 Schreibvorgang (CRUD)

1. Der Nutzer ändert eine Host-Reservierung im Web-UI.
2. Das Backend validiert die Eingabe (IPv4-Format, Duplikatsprüfung).
3. Die Änderung wird in der PostgreSQL-Datenbank persistiert.
4. **Trigger:** Nach erfolgreichem Commit wird `sync_dnsmasq_hosts()` aufgerufen.
5. Die Konfigurationsdatei wird neu generiert und geschrieben.
6. Ein `systemctl kill -s SIGHUP dnsmasq` wird ausgeführt.

### 3.2 PXE-Boot-Prozess

1. Ein Client startet via PXE/iPXE.
2. dnsmasq weist eine IP zu und verweist auf den TFTP-Server (Bootfile).
3. Der iPXE-Bootloader lädt das dynamische Menü von `/boot.ipxe` (Axum-Endpoint).
4. Das Menü wird in Echtzeit aus der Datenbank generiert.

## 4. Sicherheitskonzept

* **Berechtigungen:** Das Rust-Backend läuft unter einem eingeschränkten System-User.
* **Sudoers:** Ein spezifischer Sudoers-Eintrag erlaubt nur den Befehl `systemctl kill -s SIGHUP dnsmasq` ohne Passwort.
* **Validierung:** Strikte Typenprüfung (ipnetwork crate) verhindert fehlerhafte DHCP-Einträge.

## 5. Deployment-Struktur

```text
/etc/dnsmasq.conf           <-- Globale Einstellungen
/etc/dnsmasq.d/
    ├── 00-global.conf      <-- Statische Netzwerkeinstellungen
    └── 01-rust-hosts.conf  <-- Von IPManager generierte Leases