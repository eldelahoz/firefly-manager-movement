# TRD — Architecture technique du rapprochement bancaire

| Champ | Valeur |
|---|---|
| Projet | firefly-manager-movement |
| Version | 0.2 |
| Créé le | 2026-08-14 |
| Dernière modification | 2026-08-15 |
| PRD associé | [`2026-08-14-prd-rapprochement-bancaire.md`](./2026-08-14-prd-rapprochement-bancaire.md) |
| Statut | En cours de rédaction |

---

## 1. Vue d'ensemble

Application web self-hosted en 3 services, déployable en Docker Compose à côté de Firefly III :

```mermaid
flowchart LR
    FE["Frontend<br/>React/TS"] -->|REST| BE["Backend<br/>Rust (axum)"]
    BE -->|API v1| FF3["Firefly III<br/>(existant)"]
    BE -->|SQL| DB[("MySQL")]
```

| Service | Technologie | Rôle |
|---|---|---|
| Backend | Rust — axum + tokio, reqwest (client HTTP), Diesel (ORM, MySQL) | API REST interne, parsing CSV, moteur de rapprochement, client Firefly III |
| Frontend | React + Vite + TypeScript | SPA : upload, mapping, tableau de rapprochement, création en lot |
| Base | MySQL 8 | Persistance des batchs, mouvements, résultats, mappings CSV |

## 2. Modèle de données

```mermaid
erDiagram
    BATCHES ||--o{ MOVEMENTS : "contient"
    BATCHES ||--o{ RECONCILIATIONS : "produit"
    BATCHES }o--|| CSV_MAPPINGS : "utilise (optionnel)"

    BATCHES {
        bigint id PK
        varchar filename
        varchar file_hash
        varchar format
        char currency
        int account_ff3_id
        varchar status
        datetime created_at
    }
    MOVEMENTS {
        bigint id PK
        bigint batch_id FK
        int line_no
        date date
        bigint amount_cents
        enum direction
        text description
        json raw_json
        enum status "imported|matched|missing|created|ignored"
        bigint ff3_transaction_id "NULL"
        datetime created_at
    }
    RECONCILIATIONS {
        bigint id PK
        bigint batch_id FK
        int tolerance_days
        date window_start
        date window_end
        int matched
        int missing
        int extra
        datetime ran_at
    }
    CSV_MAPPINGS {
        bigint id PK
        varchar header_signature
        json mapping_json
        datetime created_at
    }
    DELETED_AUDIT {
        bigint id PK
        bigint ff3_transaction_id
        json snapshot_json
        datetime deleted_at
    }
```

Contraintes clés :

- Unicité `(batch_id, line_no)` sur `movements`.
- Unicité `(batch_id, ff3_transaction_id)` quand non NULL (idempotence de création, NFR-05).
- `amount_cents` : BIGINT signé ou paire (montant entier + sens) — jamais de flottant (NFR-06).
- `raw_json` : ligne brute du fichier, pour audit et re-parsing.
- Index : `(batch_id, status)`, `(batch_id, date)`.

## 3. API backend

| Méthode | Route | Rôle |
|---|---|---|
| POST | `/api/imports` | Upload fichier → création batch + parse |
| GET | `/api/imports` / `/api/imports/{id}` | Liste / détail batch |
| PUT | `/api/imports/{id}/mapping` | Enregistrer mapping CSV |
| POST | `/api/imports/{id}/reconcile` | Lancer le rapprochement |
| GET | `/api/imports/{id}/results` | Résultats en 3 groupes |
| POST | `/api/imports/{id}/apply` | Créer les manquants sélectionnés dans Firefly III |
| GET | `/api/firefly/accounts` | Proxy lecture des comptes d'actif |
| QUERY | `/api/firefly/accounts/{id}/transactions` | Proxy lecture des mouvements d'un compte sur une période — filtres en body : `{start, end}` (RF-14) |
| DELETE | `/api/firefly/transactions/{id}` | Suppression d'un doublon avec snapshot d'audit (RF-15) |

Notes :

- `POST /apply` est idempotent par `(batch, movement)` : avant chaque création, vérifier l'état en base ; ignorer les mouvements déjà `created`.
- Erreurs de l'API Firefly III lors d'un `apply` en lot : traitement par mouvement, le lot continue, rapport final par mouvement (succès/échec/raison).
- **Méthode QUERY (RFC 9295)** : utilisée uniquement pour les routes à filtres complexes en body (lectures avec critères de recherche). Les récupérations simples par ID ou listes complètes restent en GET. Le backend traduit `QUERY` → `GET` lors de l'appel à l'API Firefly III (qui reste en GET). Nécessite d'enregistrer la méthode explicitement dans axum (`Route::on("QUERY", handler)` ou macro équivalente).

## 4. Intégration Firefly III (API v1)

| Usage | Endpoint |
|---|---|
| Auth | Header `Authorization: Bearer <token>` (personal access token ; OAuth2 supporté aussi) |
| Comptes | `GET /api/v1/accounts?type=asset` |
| Transactions | `GET /api/v1/accounts/{id}/transactions?start=..&end=..` (pagination exhaustive) — appelé par le backend en traduisant notre `QUERY` interne |
| Création | `POST /api/v1/transactions` |
| Suppression | `DELETE /api/v1/transactions/{id}` |
| Santé | `GET /api/v1/about` |

Configuration backend (variables d'environnement) :

| Variable | Description |
|---|---|
| `FF3_URL` | URL de l'instance Firefly III |
| `FF3_TOKEN` | Token API (jamais exposé au frontend, NFR-02) |
| `DATABASE_URL` | Chaîne de connexion MySQL |
| `MATCH_TOLERANCE_DAYS` | Tolérance de date par défaut (défaut 0) |

Le frontend ne parle jamais directement à Firefly III : tout passe par le backend.

## 5. Parsing des fichiers

- **CSV** : détection du délimiteur et de l'encodage ; si les en-têtes correspondent à un mapping mémorisé (`csv_mappings.header_signature`, hash des en-têtes normalisés), application automatique ; sinon mapping manuel requis. Les fichiers XLSX doivent être convertis en CSV par l'utilisateur avant import (support natif XLSX/OFX prévu en V2).
- Lignes invalides : conservées avec erreur attachée, exposées dans l'aperçu (RF-03).

## 6. Moteur de rapprochement

1. Charger les mouvements du batch (`status != ignored`).
2. Récupérer les transactions Firefly III du compte cible sur `[min_date - fenêtre, max_date + fenêtre]`, pagination exhaustive.
3. Indexer les deux jeux par clé `(montant_cents, sens)`.
4. Pour chaque clé, apparier chronologiquement 1-pour-1 dans la tolérance de date ; les transactions Firefly III surnuméraires face à un même mouvement bancaire sont marquées `duplicate` (RF-15).
5. Produire les 3 groupes ; persister une ligne `reconciliations` + mise à jour des statuts des mouvements.

Complexité cible : O(n log n) sur le relevé ; 1 000 mouvements < 5 s hors latence API (NFR-04).

## 7. Déploiement

Docker Compose : 3 services (`backend`, `frontend`, `mysql`) + réseau partagé avec l'instance Firefly III existante. Migrations de schéma gérées par Diesel (`diesel migration run`) au démarrage du backend.

Note ORM : Diesel est synchrone par défaut ; l'intégration avec axum/tokio se fait via `diesel-async` (pool MySQL async) pour ne pas bloquer le runtime.

## 8. Décisions ouvertes

| # | Sujet | Options |
|---|---|---|
| D1 | Framework backend Rust | axum (recommandé) vs actix-web |
| D2 | Représentation des montants | BIGINT centimes (recommandé) vs DECIMAL(19,4) |
| D3 | Stratégie de re-rapprochement après `apply` | complet vs incrémental sur les mouvements créés |
