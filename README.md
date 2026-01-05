# **IP-Management-System für Kea DHCP und PostgreSQL**

## **Überblick**

Das IP-Management-System verwaltet IP-Adressen, DHCP-Leases und Hosts in einer Netzwerkumgebung, die mit dem Kea DHCP-Server und einer PostgreSQL-Datenbank integriert ist. Es bietet eine Webanwendung zur Verwaltung von Hosts, Subnetzen und DHCP-Pools, um Netzwerkadministratoren zu unterstützen.

### **Technologien:**

* **Backend:** Rust, Axum (Webframework), SQLx (PostgreSQL-Interaktion)
* **Frontend:** Tera Templates, HTML/CSS
* **Datenbank:** PostgreSQL
* **DHCP:** Kea DHCP
* **Session-Management:** Tower Sessions (mit PostgreSQL-Store)

### **Hauptfunktionen:**

1. **Host-Management:**

   * Hinzufügen, Bearbeiten und Anzeigen von Hosts.
   * Validierung von IP-Adressen (IPv4) und MAC-Adressen.
   * Zuordnung von Hosts zu Subnetzen und LAN-Dosen.

2. **Subnetz-Management:**

   * Verwaltung von Subnetzen und deren zugehörigen DHCP-Pools.
   * Prüfung von DHCP-Poolbereichen (Start/Ende) auf Gültigkeit.

3. **DHCP-Pool-Management:**

   * Prüfung auf gültige IP-Adressen innerhalb eines Subnetzes.
   * Sicherstellung, dass IP-Bereiche korrekt eingegeben werden.

4. **Fehler und Validierungen:**

   * IP-Adressen und Subnetzzugehörigkeit müssen geprüft werden, um ungültige Daten zu verhindern.
   * Fehlende Validierungen beim Anlegen von Hosts und Subnetzen wurden hinzugefügt.

---

## **Systemarchitektur**

### **1. Backend (Rust mit Axum und SQLx)**

* Das Backend ist in **Rust** implementiert und nutzt **Axum** als Webframework.
* **SQLx** wird zur Interaktion mit der PostgreSQL-Datenbank verwendet, wobei asynchrone Datenbankoperationen durchgeführt werden.
* Die Kommunikation mit der Kea DHCP-API erfolgt über HTTP-Requests.

### **2. PostgreSQL-Datenbank**

* Die Datenbank speichert alle relevanten Informationen über Hosts, Subnetze, DHCP-Pools und deren Zuordnungen.
* **Tabellen:**

  * **Hosts:** Enthält Informationen zu Hosts, einschließlich ihrer IP-Adressen und MAC-Adressen.
  * **Subnetze:** Enthält Subnetz-Informationen, einschließlich CIDR und zugehöriger DHCP-Pool-Bereiche.
  * **DHCP-Pools:** Verwalten IP-Bereiche für DHCP-Leases.

### **3. DHCP-Integration (Kea DHCP)**

* Kea DHCP wird zur Zuweisung von IP-Adressen in der Netzwerkumgebung verwendet. Das System kann DHCP-Pools und -Subnetze konfigurieren und überwachen.
* Die Kea DHCP-Konfiguration wird über JSON-Dateien verwaltet, die durch die Webanwendung erstellt und angepasst werden können.

---

## **Installation und Setup**

### **Voraussetzungen**

* **Rust** und **Cargo** (Rust’s Paketmanager und Build-Tool)
* **PostgreSQL** für die Datenbank
* **Kea DHCP** für den DHCP-Server
* **Docker** (optional, für schnelle Testumgebungen)

### **1. PostgreSQL-Datenbank einrichten**

* Erstelle eine PostgreSQL-Datenbank:

  ```bash
  createdb ipmanager_db
  ```

* Lege die erforderlichen Tabellen an:

  ```sql
  CREATE TABLE hosts (
      id SERIAL PRIMARY KEY,
      ip_address INET NOT NULL,
      mac_address VARCHAR(17) NOT NULL,
      hostname VARCHAR(255) UNIQUE NOT NULL
  );

  CREATE TABLE subnets (
      id SERIAL PRIMARY KEY,
      cidr VARCHAR(18) NOT NULL,
      dns_zone VARCHAR(255),
      reverse_zone VARCHAR(255)
  );

  CREATE TABLE dhcp_pools (
      id SERIAL PRIMARY KEY,
      subnet_id INT REFERENCES subnets(id),
      start_ip INET NOT NULL,
      end_ip INET NOT NULL
  );
  ```

### **2. Abhängigkeiten installieren**

* Füge die Abhängigkeiten in `Cargo.toml` hinzu:

  ```toml
  [dependencies]
  axum = "0.5"
  sqlx = { version = "0.5", features = ["postgres", "runtime-tokio-native-tls"] }
  tokio = { version = "1", features = ["full"] }
  tower-sessions = "0.2"
  ipnetwork = "0.18"
  regex = "1"
  ```

* Installiere alle Abhängigkeiten:

  ```bash
  cargo build
  ```

### **3. Kea DHCP Integration**

* Konfiguriere den Kea DHCP-Server, um das IP-Management-System zu nutzen. Das System stellt sicher, dass die generierten Konfigurationen für Kea korrekt sind und dass Subnetze und DHCP-Pools dynamisch verwaltet werden.

---

## **Hauptfunktionen und Endpunkte**

### **1. Host-Management**

* **POST /hosts/create**: Erstellt einen neuen Host. Überprüft, ob die IP-Adresse und MAC-Adresse gültig sind und ob sie bereits existieren.
* **PUT /hosts/update**: Aktualisiert einen bestehenden Host. Überprüft die IP-Adresse und MAC-Adresse auf Gültigkeit.
* **GET /hosts/{id}**: Zeigt die Details eines Hosts an.
* **GET /hosts**: Listet alle Hosts auf.

### **2. Subnetz-Management**

* **POST /subnets/create**: Erstellt ein neues Subnetz und fügt DHCP-Pools hinzu.
* **PUT /subnets/update**: Aktualisiert ein bestehendes Subnetz.
* **GET /subnets/{id}**: Zeigt die Details eines Subnetzes an.
* **GET /subnets**: Listet alle Subnetze auf.

### **3. DHCP-Pool-Management**

* **POST /dhcp-pools/create**: Erstellt einen neuen DHCP-Pool.
* **PUT /dhcp-pools/update**: Aktualisiert einen bestehenden DHCP-Pool.
* **GET /dhcp-pools/{id}**: Zeigt die Details eines DHCP-Pools an.

### **4. Fehlermeldungen und Validierungen**

* **IP-Adressvalidierung:** Überprüft, ob die IP-Adresse im gültigen IPv4-Format vorliegt.
* **MAC-Adressvalidierung:** Stellt sicher, dass die MAC-Adresse im richtigen Format vorliegt.
* **Duplicate-Check:** Vor der Speicherung eines Hosts wird geprüft, ob die IP-Adresse, MAC-Adresse oder der Hostname bereits existieren.

---

## **Tests**

### **Unit-Tests**

* **IP- und MAC-Validierung:** Alle Validierungsfunktionen für IP- und MAC-Adressen werden mit verschiedenen gültigen und ungültigen Werten getestet.
* **Duplicate-Check:** Sicherstellung, dass Duplikate bei der Host-Erstellung oder -Aktualisierung korrekt erkannt und abgelehnt werden.

### **Integrationstests**

* **Datenbank-Integration:** Die Integrationstests prüfen, ob Daten korrekt in der PostgreSQL-Datenbank gespeichert und abgerufen werden. Sie testen auch, ob Duplikate und falsche Fremdschlüsselverletzungen richtig behandelt werden.
* **Transaktionen:** Jeder Test läuft in einer Transaktion, die nach dem Test zurückgerollt wird, um die Datenbank in einen sauberen Zustand zu versetzen.

### **Test-Befehl:**

```bash
cargo test
```

---

## **Fazit**

Das IP-Management-System ist ein robustes Werkzeug zur Verwaltung von Hosts, Subnetzen und DHCP-Pools in einer Netzwerkumgebung, die mit Kea DHCP und PostgreSQL integriert ist. Es bietet umfangreiche Funktionen für die IP-Adressen- und MAC-Adressen-Validierung, Duplikatprüfungen sowie eine nahtlose Integration mit dem Kea DHCP-Server.

### **Nächste Schritte:**

* Bereitstellung in einer Produktionsumgebung.
* Weiterführende Tests und Optimierungen für die Datenbankoperationen.
* Dokumentation und Schulung für Benutzer des Systems.