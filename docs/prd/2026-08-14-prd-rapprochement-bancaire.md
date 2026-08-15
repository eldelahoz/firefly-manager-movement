# PRD — Rapprochement bancaire pour Firefly III

| Champ | Valeur |
|---|---|
| Projet | firefly-manager-movement |
| Version | 0.1 (draft initial) |
| Date | 2026-08-14 |
| Auteur | A. Delahoz |
| Statut | En cours de rédaction |

---

## 1. Problème et contexte

Firefly III est un gestionnaire de finances personnelles self-hosted largement adopté. Il permet de suivre comptes, budgets, catégories et transactions. Cependant, il ne propose **aucun module natif de rapprochement bancaire** : il est impossible d'importer un relevé de transactions bancaires, de le comparer aux transactions déjà enregistrées, de visualiser les écarts et de créer automatiquement les transactions manquantes.

Conséquences pour l'utilisateur :

- Saisie manuelle fastidieuse ou double saisie (banque + Firefly III).
- Aucune garantie d'exhaustivité : des transactions bancaires peuvent manquer dans Firefly III sans que personne ne s'en aperçoive.
- Aucune vue comparative : impossible de savoir rapidement « ce qui est dans la banque mais pas dans mon budget ».

### Énoncé du problème

> En tant qu'utilisateur de Firefly III, je veux importer mes relevés bancaires, les comparer aux transactions existantes, voir clairement ce qui correspond / ce qui manque, et créer en un clic les transactions manquantes — afin que mon suivi financier soit exhaustif et sans double saisie.

---

## 2. Objectifs

| # | Objectif | Mesure de succès |
|---|---|---|
| O1 | Importer des relevés bancaires (CSV/OFX) sans configuration bancaire | 100 % des fichiers de test importés avec succès |
| O2 | Comparer automatiquement relevé vs Firefly III | ≥ 95 % des correspondances exactes détectées sans intervention |
| O3 | Rendre les écarts visibles et actionnables | L'utilisateur identifie les manquants en < 30 s après import |
| O4 | Créer les transactions manquantes dans Firefly III à la demande | 0 transaction créée sans action explicite de l'utilisateur |

### Non-objectifs (v1)

- Connexion directe aux banques via API (GoCardless/Nordigen, Tink, etc.) — architecture prévue pour l'ajouter ensuite.
- Synchronisation bidirectionnelle ou modification/suppression de transactions existantes dans Firefly III.
- Rapprochement multi-utilisateurs / multi-foyers.
- Catégorisation automatique par ML des transactions importées.
- Remplacement du Data Importer officiel de Firefly III (outil complémentaire, pas concurrent).

---

## 3. Personas et cas d'utilisation

### Persona principal : « Alex », utilisateur self-hosted

Utilise Firefly III en Docker chez lui. Télécharge chaque mois un CSV/OFX depuis le site de sa banque. Veut vérifier que tout est bien tracé sans ressaisir.

### Cas d'utilisation

| ID | Cas | Description |
|---|---|---|
| CU-1 | Importer un relevé | Alex upload un fichier CSV ou OFX ; le système le parse, détecte le format et affiche un aperçu des mouvements. |
| CU-2 | Mapper les colonnes (CSV) | Si le CSV n'est pas reconnu automatiquement, Alex mappe colonnes → champs (date, montant, description, compte). |
| CU-3 | Lancer le rapprochement | Alex choisit le compte Firefly III cible et lance la comparaison. |
| CU-4 | Consulter les résultats | Vue en 3 groupes : rapprochés / manquants dans Firefly III / présents dans Firefly III mais absents du relevé. |
| CU-5 | Créer les manquants | Alex sélectionne tout ou partie des manquants, choisit compte de contrepartie/catégorie, et valide la création dans Firefly III. |
| CU-6 | Historique des imports | Alex retrouve ses imports passés (batchs) et leur état. |

---

## 4. Flux fonctionnel

Pipeline en 3 étapes : **Importer → Rapprocher → Appliquer**.

```
[Fichier CSV/OFX]
      │ upload
      ▼
  (1) IMPORT ──► parsing + validation ──► aperçu + mapping
      │                                        │
      │            batch immuable persisté (MySQL)
      ▼
  (2) RAPPROCHEMENT ──► appel API Firefly III (transactions du compte cible,
      │                  fenêtre de date) ──► moteur de matching
      │                                        │
      │            résultats persistés : matched / missing / extra
      ▼
  (3) APPLICATION ──► sélection utilisateur ──► création via API Firefly III
                       ──► re-rapprochement pour confirmer
```

### États d'un mouvement importé

| État | Signification |
|---|---|
| `imported` | Présent dans le batch, pas encore rapproché |
| `matched` | Correspondance trouvée dans Firefly III |
| `missing` | Aucune correspondance : candidat à la création |
| `created` | Créé dans Firefly III par l'utilisateur via l'app |
| `ignored` | Écarté manuellement par l'utilisateur (ex. virement interne) |

### Règles de rapprochement (matching)

Critères de correspondance :

1. **Montant** : égalité exacte (comparaison en centimes/entiers, jamais en flottants).
2. **Sens** : retrait vs dépôt doit correspondre.
3. **Date** : égalité avec **tolérance configurable** (défaut ±0 jour ; options ±1, ±2, ±3 jours).
4. **Compte** : le rapprochement est toujours limité à un compte Firefly III cible.

Paramètres configurables par l'utilisateur :

- Tolérance de date (jours).
- Fenêtre de recherche autour du relevé (défaut : dates du relevé ±7 jours).

Cas particuliers :

- **Correspondances multiples** (deux mouvements identiques montant+date) : matching 1-pour-1 par ordre chronologique ; les excédents restent `missing`/`extra`.
- **Frais bancaires / intérêts** : aucun traitement spécial en v1 ; ils apparaissent comme `missing`.

### Groupes de résultat

| Groupe | Contenu | Action possible |
|---|---|---|
| **Rapprochés** (`matched`) | Mouvements présents des deux côtés | Aucune (lecture seule) |
| **Manquants** (`missing`) | Dans le relevé, absents de Firefly III | Créer dans Firefly III (individuel ou en lot) |
| **Extras** (`extra`) | Dans Firefly III, absents du relevé | Informatif uniquement en v1 |

### Création des transactions manquantes

- L'utilisateur sélectionne les mouvements à créer (checkbox + « tout sélectionner »).
- Champs requis avant création : compte de contrepartie (compte de dépenses/recettes ou compte d'actif selon le sens). Champs optionnels : catégorie, budget, notes.
- La création appelle l'API Firefly III (`POST /api/v1/transactions`) avec : type, montant, date, description (issue du relevé), compte source/destination, et un tag/notes traçant l'origine (nom du fichier + id de batch).
- Après création, le mouvement passe à l'état `created` et un re-rapprochement incrémental confirme la correspondance.
- **Aucune création n'est faite sans action explicite.** Pas de mode automatique en v1.

---

## 5. Exigences fonctionnelles

| ID | Exigence | Priorité |
|---|---|---|
| RF-01 | Le système permet l'upload de fichiers CSV et OFX (taille max 10 Mo). | P0 |
| RF-02 | Le système parse l'OFX sans configuration. | P0 |
| RF-03 | Le système parse le CSV avec mapping manuel des colonnes ; il mémorise le mapping par signature de fichier (en-têtes). | P0 |
| RF-04 | Le système détecte et signale les lignes invalides (date illisible, montant vide) sans bloquer l'import ; les lignes valides sont conservées. | P0 |
| RF-05 | Le système liste les comptes d'actif Firefly III pour choisir le compte cible du rapprochement. | P0 |
| RF-06 | Le système récupère les transactions du compte cible via l'API Firefly III sur la fenêtre de date. | P0 |
| RF-07 | Le moteur de rapprochement applique les règles de la section 4 avec tolérance de date configurable. | P0 |
| RF-08 | Les résultats sont présentés en 3 groupes (rapprochés / manquants / extras) avec mise en évidence visuelle des écarts. | P0 |
| RF-09 | L'utilisateur peut créer en lot les transactions manquantes dans Firefly III. | P0 |
| RF-10 | Chaque batch d'import est immuable et consultable a posteriori (historique). | P1 |
| RF-11 | L'utilisateur peut ignorer manuellement un mouvement (`ignored`). | P1 |
| RF-12 | Les transactions créées portent une trace d'origine (tag ou note : fichier + batch). | P1 |
| RF-13 | Le système gère plusieurs devises : la devise du relevé doit correspondre à celle du compte cible, sinon avertissement bloquant. | P1 |
| RF-14 | L'utilisateur peut re-lancer un rapprochement sur un batch existant. | P1 |

## 6. Exigences non fonctionnelles

| ID | Exigence |
|---|---|
| NFR-01 | **Architecture** : application web indépendante ; backend Rust, frontend React, base MySQL. Déployable en Docker Compose à côté de Firefly III. |
| NFR-02 | **Sécurité** : le token d'API Firefly III est stocké côté backend uniquement (variables d'environnement / coffre local), jamais exposé au navigateur. Aucune donnée bancaire n'est envoyée à un service tiers. |
| NFR-03 | **Confidentialité** : self-hosted uniquement ; aucune télémétrie. |
| NFR-04 | **Performance** : rapprochement d'un relevé de 1 000 mouvements < 5 s (hors latence API Firefly III). |
| NFR-05 | **Fiabilité** : les créations dans Firefly III sont idempotentes par batch+mouvement (pas de double création en cas de retry). |
| NFR-06 | **Montants** : manipulés en entiers (centimes) ou decimal exact, jamais en flottants. |
| NFR-07 | **Observabilité** : logs structurés des imports, rapprochements et créations. |
| NFR-08 | **Internationalisation** : interface en français et espagnol (défaut : français) ; montants et dates formatés selon la locale. |

---

## 7. Stack technique (vue d'ensemble)

| Couche | Choix |
|---|---|
| Backend | Rust |
| Frontend | React |
| Base de données | MySQL |
| Déploiement | Docker Compose, à côté de Firefly III |

Le détail technique (modèle de données, endpoints, intégration API Firefly III) est décrit dans le TRD : [`2026-08-14-trd-architecture-technique.md`](./2026-08-14-trd-architecture-technique.md).

---

## 8. Métriques de succès

| Métrique | Cible |
|---|---|
| Taux de lignes parsées avec succès sur fichiers réels | ≥ 98 % |
| Précision du matching (correspondances exactes détectées) | ≥ 95 % |
| Faux positifs de matching | < 1 % |
| Temps pour passer de l'upload à la liste des manquants | < 30 s |
| Transactions créées sans action utilisateur | 0 |

---

## 9. Risques et mitigations

| Risque | Impact | Mitigation |
|---|---|---|
| Formats CSV très hétérogènes selon les banques | Import en échec | Mapping manuel + mémorisation par signature d'en-têtes ; bibliothèque de formats connus extensible |
| Pagination/volume de l'API Firefly III sur longues fenêtres | Rapprochement lent/incomplet | Pagination exhaustive côté client API ; fenêtre de date bornée |
| Doublons côté banque (même montant/date légitimes) | Matching 1-pour-1 incorrect | Appariement chronologique + revue manuelle des excédents |
| Création accidentelle en lot | Pollution de Firefly III | Confirmation explicite, traçabilité (tag batch), jamais d'auto-création |
| Token API compromis | Accès complet à Firefly III | Token côté backend uniquement, rotation documentée |

---

## 10. Hors périmètre / futur

- **V2** : connexion bancaire directe (GoCardless/Nordigen) via le même pipeline (le connecteur remplace l'upload).
- **V2** : règles de matching par description (fuzzy), suggestions de contrepartie.
- **V2** : gestion du groupe `extra` (marquer comme attendu, alertes).
- **V3** : catégorisation automatique, rapports d'exhaustivité, multi-comptes en un rapprochement.

---

## 11. Glossaire

| Terme | Définition |
|---|---|
| Rapprochement (reconciliation) | Comparaison systématique de deux jeux de transactions pour identifier correspondances et écarts |
| Batch | Ensemble immuable des mouvements issus d'un fichier importé |
| Matching | Règles déterminant si un mouvement du relevé correspond à une transaction Firefly III |
| Manquant (`missing`) | Mouvement présent dans le relevé bancaire mais absent de Firefly III |
| Extra (`extra`) | Transaction présente dans Firefly III mais absente du relevé |
| Tolérance de date | Écart maximal en jours accepté entre deux dates pour considérer une correspondance |
