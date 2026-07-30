[PRD]
# PRD: Cohérence du cycle de vie Arthur Skills

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-30 | Arthur Jean | Correctif du contrat de provenance, de planification, d'adoption et de convergence du receipt |
| 1.1 | 2026-07-30 | Arthur Jean | EP-002: précision de la classification d'un chemin absent et du receipt commit lié à une réécriture de lock vérifié |
| 1.2 | 2026-07-30 | Arthur Jean | EP-003: unification du résultat périmé sur `stale_lifecycle_decision` et extension de la revalidation à toute claim déjà possédée |

## Problem Statement

1. Sur un état utilisateur réel observé avec le catalogue 0.2.1, `plan` rapporte 10 assets `adoptable` et 4 `drifted`, `install --dry-run` et `update --dry-run` ne conservent que 7 assets `adoptable`, puis `adopt --dry-run` échoue avec `MissingLegacyEntry`. Une même situation produit donc trois décisions incompatibles.
2. Le planner classe aujourd'hui un contenu identique non possédé comme `adoptable` sans preuve externe, alors que le moteur d'adoption exige une entrée correspondante dans un lock Vercel Skills v3. L'interface recommande ainsi une commande qui ne peut pas satisfaire sa propre précondition.
3. La transition d'import peut construire une baseline de receipt à partir de la seule présence d'un chemin désiré. Elle peut alors transformer un asset étranger en asset possédé sans receipt antérieur ni preuve legacy vérifiée.
4. Les changements limités au receipt ne sont pas représentés comme une opération de cycle de vie. Une sortie anticipée `already_current` peut empêcher l'enregistrement d'un catalogue, de hashes ou de métadonnées déjà convergents sur disque.
5. L'UI, les sorties humaines et le JSON ne conservent pas toute la provenance ayant conduit à la décision. Un diagnostic d'adoption peut perdre le `source_id` et le chemin fautif, ce qui empêche une remédiation ciblée.

**Why now:** le PRD parent est marqué `DONE`, mais l'ajout récent de nouveaux skills a activé une contradiction présente depuis l'introduction séparée du planner générique et de l'adoption Vercel v3. Le CLI 0.2.1 peut maintenant bloquer une mise à jour normale tout en recommandant une action inexécutable. Corriger le contrat avant un nouvel élargissement du catalogue réduit le risque de revendiquer des fichiers étrangers et évite de stabiliser une sémantique JSON incohérente.

## Overview

Ce PRD corrige le cycle de vie existant sans transformer `arthur-skills` en gestionnaire de packages généraliste. Une requête de cycle de vie produit une seule `LifecycleDecision` déterministe à partir du catalogue désiré, du filesystem observé, du receipt courant, du lock legacy vérifié et des providers demandés. Cette décision contient le plan d'assets, les preuves d'ownership, les vérifications pré-commit, le receipt projeté, les diagnostics, l'applicabilité et le résumé. `plan`, les modes dry-run, Ratatui, plain, JSON et l'exécuteur consomment cette même décision sans recalcul métier.

L'ownership suit une règle fermée. Un chemin est possédé parce qu'un receipt courant le prouve, ou adoptable parce qu'une entrée legacy vérifiée prouve son `source_id` et que chaque asset associé est strictement conforme. Un chemin absent peut être créé. Un chemin identique mais dépourvu de preuve reste non possédé et devient `conflict` avec le code `matching_unmanaged_without_proof`. Un chemin divergent non possédé reste `conflict`. Aucun hash identique, nom de dossier ou présence filesystem ne constitue à lui seul une preuve d'ownership.

La transaction continue d'utiliser le verrou, le staging, le journal, les snapshots et le rollback existants. Elle ajoute une revalidation de chaque claim d'ownership avant la première mutation et, pour les assets revendiqués sans mutation de contenu, immédiatement avant le commit du receipt. Le receipt projeté devient une opération visible lorsque sa valeur sémantique change, même si tous les assets sont déjà égaux au catalogue.

Ce PRD amende les exigences contradictoires du PRD parent concernant les contenus identiques non possédés. Les clauses de sûreté FR-20 et d'adoption Vercel v3 prévalent: sans preuve receipt ou legacy vérifiée, `adopt` n'est pas une remédiation valide.

## Goals

| Goal | Month-1 Target | Month-6 Target |
|------|---------------|----------------|
| Unifier la décision de cycle de vie | 100% des fixtures partagées produisent le même plan normalisé pour `plan`, install/update dry-run, UI et apply | 0 régression de parité sur chaque release |
| Garantir la provenance de l'ownership | 100% des assets écrits dans un receipt possèdent une preuve `receipt`, `verified_legacy` ou `created_in_transaction` | 0 asset étranger revendiqué dans la matrice multi-plateforme |
| Rendre les remédiations exécutables | 0 diagnostic ne recommande `adopt` sans candidat legacy vérifié | 0 issue confirmée de boucle `plan -> adopt -> blocked` |
| Converger les métadonnées | 100% des changements receipt-only produisent une opération visible et un second run no-op | 0 receipt périmé après une transaction réussie |
| Fermer la fenêtre de claim concurrent | 100% des claims sans écriture de contenu sont revalidés avant commit | 100% des injections de changement terminent sans faux ownership |

## Target Users

### Utilisateur avec une installation existante

- **Role:** utilisateur de Claude Code, Codex ou des deux ayant déjà un catalogue Arthur, un receipt antérieur, un lock Vercel Skills ou des fichiers homonymes.
- **Behaviors:** exécute `plan`, `install`, `update` et `adopt`, conserve des skills personnels dans les mêmes racines et attend une mise à jour offline.
- **Pain points:** voit des comptes différents selon la commande, reçoit une remédiation `run adopt` qui échoue, et ne sait pas quel asset manque de preuve.
- **Current workaround:** compare manuellement receipt, lock et filesystem, puis déplace des dossiers avant de relancer le CLI.
- **Success looks like:** obtient une décision identique sur toutes les surfaces, un chemin exact pour chaque blocker et aucune revendication d'un asset sans preuve.

### Mainteneur du CLI

- **Role:** mainteneur qui publie le catalogue et fait évoluer le moteur de cycle de vie.
- **Behaviors:** ajoute ou modifie des skills, maintient les contrats Rust, les fixtures filesystem, les sorties JSON et les scénarios process.
- **Pain points:** trois pipelines produisent des transitions différentes, les tests de planner et d'adoption prouvent séparément des hypothèses incompatibles, et un nouveau skill peut révéler tardivement une collision.
- **Current workaround:** inspecte les transitions intermédiaires et reproduit l'état utilisateur avec plusieurs dry-runs.
- **Success looks like:** une fixture de décision suffit à prouver planification, rendu et application, avec une provenance typée vérifiable à chaque frontière.

## Research Findings

Key findings that informed this PRD:

### Competitive Context

- [GNU Stow](https://www.gnu.org/software/stow/manual/stow.html) sépare le préflight de conflits de la mutation, refuse les collisions non possédées, expose `--simulate` et réserve `--adopt` à une action explicite dont le caractère destructif est documenté.
- [NixOS](https://nixos.org/manual/nixos/stable/) et le [rollback des profils Nix](https://nix.dev/manual/nix/stable/command-ref/new-cli/nix3-profile-rollback) utilisent des objets immuables et des générations versionnées pour rendre la transition et le rollback explicites.
- [dpkg-query](https://manpages.debian.org/bookworm/dpkg/dpkg-query.1.en.html) fonde l'ownership sur une base interrogeable par chemin. La [simulation APT](https://manpages.debian.org/bookworm/apt/apt-get.8) montre aussi qu'un preview sans verrou peut devenir périmé avant l'application.
- [Terraform import](https://developer.hashicorp.com/terraform/cli/import) exige un mapping explicite entre objet réel et adresse d'état. [Terraform plan](https://developer.hashicorp.com/terraform/cli/commands/plan) distingue les plans spéculatifs des plans sauvegardés exécutables et documente leur obsolescence possible.
- [Homebrew](https://docs.brew.sh/Manpage.html) sépare le preview de cleanup de son application forcée et avertit lorsque des suppressions peuvent toucher des fichiers partagés.
- **Market gap:** le besoin n'est pas un nouveau gestionnaire généraliste, mais une décision locale unique qui associe chaque asset à une provenance vérifiable et conserve cette preuve jusqu'au receipt commit.

### Best Practices Applied

- Calculer tous les conflits et toutes les preuves avant mutation.
- Ne jamais inférer l'ownership depuis un nom, un chemin existant ou un contenu identique.
- Présenter et appliquer le même artefact de décision, puis revalider ses préconditions au moment de la mutation.
- Séparer adoption explicite, création, réconciliation d'un receipt existant et collision étrangère.
- Représenter les changements de control plane, dont le receipt, dans le même plan que les changements filesystem.
- Préserver un diagnostic machine-readable avec code stable, destination exacte et remédiation exécutable.

*Full research sources are linked above. Codebase evidence is captured in the implementation boundaries and acceptance criteria below.*

## Assumptions & Constraints

### Assumptions (to validate)

- Le receipt v1 contient assez d'identité de chemin, de hash, de mode, de type et de cible pour prouver l'ownership antérieur sans migration destructive.
- Une entrée Vercel Skills v3 vérifiée peut prouver son skill canonique et uniquement les activations exactes que son `source_id` permet de dériver et de valider; elle ne prouve pas automatiquement des agents ou supports absents de son modèle.
- L'ajout des champs JSON optionnels `ownership_basis` et `source_id` reste backward-compatible pour les consommateurs v1 qui ignorent les champs inconnus. Les fixtures de schéma doivent le prouver avant publication.
- Les snapshots et préconditions existants peuvent être réutilisés pour revalider les claims sans nouvelle dépendance.
- Le coût de l'inspection legacy intégrée à la décision reste dans le budget P95 de planification existant.

### Hard Constraints

- Aucun chemin préexistant absent du receipt ne peut être revendiqué sans preuve legacy vérifiée asset par asset.
- Un contenu identique sans preuve est `conflict`, jamais `adoptable`.
- `adopt` reste limité au transfert d'une installation Vercel Skills v3 vérifiée. V1 n'ajoute aucune adoption générique par confirmation utilisateur.
- Pour une même requête, le même catalogue, le même receipt, le même lock et le même filesystem, toutes les surfaces consomment une décision métier unique.
- Apply ne recalcule pas une seconde transition. Il revalide les fingerprints de la décision reçue.
- `plan` et tous les dry-runs restent strictement read-only.
- Les codes de sortie publics 0, 2, 3, 4, 5, 130 et 143 restent inchangés.
- Le schéma JSON reste en major v1 et ne retire ni ne renomme aucun champ public existant.
- Aucun runtime async, backend, service, accès réseau ou nouvelle crate n'est introduit.
- Les tests utilisent exclusivement des HOME temporaires. Ils ne lisent ni ne mutent les installations live sous `/home/arthur/.agents`, `/home/arthur/.claude` ou `/home/arthur/.codex`.
- Les invariants transactionnels, de rollback, de récupération et de permissions du PRD parent restent applicables.

## Quality Gates

These commands must pass for every user story:

- `cargo fmt --all -- --check` - vérifie le format Rust sans modifier les fichiers.
- `cargo check --workspace --all-targets --all-features` - compile toutes les surfaces du workspace.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used` - refuse les warnings et les unwrap/expect de production non motivés.
- `cargo test --workspace --all-targets --all-features` - exécute unités, contrats lifecycle, intégrations filesystem, renderers et scénarios process.
- `cargo llvm-cov --workspace --all-features --fail-under-regions 90` - exige au moins 90% de couverture de régions LLVM.
- `cargo deny check` - vérifie advisories, licences, sources et duplications.

Aucun navigateur n'est requis. Les stories Ratatui utilisent `TestBackend`, les snapshots textuels et les scénarios pseudo-terminal existants.

## Epics & User Stories

### EP-001: Provenance et ownership vérifiables

Remplacer l'inférence depuis le filesystem par une provenance fermée, puis borner l'adoption et l'import aux seuls assets dont l'ownership est démontrable.

**Definition of Done:** chaque asset planifié expose une base d'ownership déterministe; aucun import, adoption ou receipt projeté ne revendique un chemin depuis sa seule présence ou égalité de contenu.

#### US-001: Modéliser la provenance de chaque asset

**Description:** As a mainteneur, I want associer chaque asset observé à une provenance fermée so that toute décision d'ownership repose sur une preuve inspectable.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** None

**Acceptance Criteria:**

- [ ] Given un asset présent dans un receipt valide, when la décision est calculée, then sa provenance vaut `receipt` et référence le record ainsi que son fingerprint attendu.
- [ ] Given un asset associé à une entrée legacy v3 validée, when contenu, type, mode et cible éventuelle correspondent, then sa provenance vaut `verified_legacy` et conserve le `source_id` ainsi que le fingerprint du lock.
- [ ] Given un chemin absent de toute preuve, when il est observé, then sa provenance vaut `none` même si son nom et ses bytes correspondent au catalogue.
- [ ] Given un asset créé dans la transaction projetée, when le receipt est construit, then sa claim d'ownership vaut `created_in_transaction` et référence l'opération créatrice.
- [ ] Given les mêmes entrées dans un ordre filesystem différent, when les provenances sont sérialisées, then leur ordre et leurs identifiants stables sont identiques.
- [ ] Given un receipt corrompu, futur ou dont l'identité de racine ne correspond pas, when la provenance est résolue, then la décision devient bloquée et aucune fallback depuis le contenu observé n'est autorisée.

#### US-002: Classer les chemins non prouvés sans faux adoptable

**Description:** As an utilisateur, I want distinguer un asset adoptable d'un asset seulement identique so that la remédiation proposée puisse réellement réussir.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-001

**Acceptance Criteria:**

- [ ] Given un chemin absent, when le catalogue le désire, then le plan le classe `create`.
- [ ] Given un chemin non possédé dont type, contenu, mode et cible correspondent au désiré, when le plan est calculé, then il le classe `conflict` avec le diagnostic `matching_unmanaged_without_proof`.
- [ ] Given un chemin non possédé divergent, when le plan est calculé, then il reste `conflict` avec la raison exacte de divergence.
- [ ] Given un chemin prouvé par le receipt et égal au désiré, when le plan est calculé, then il est `noop` même si les métadonnées de catalogue du receipt doivent converger.
- [ ] Given un chemin prouvé par le receipt qui diffère à la fois de son fingerprint attendu et du catalogue désiré, when le plan est calculé, then il est `drifted` et aucune mise à jour destructive n'est planifiée.
- [ ] Given un asset identique sans preuve, when les remédiations sont rendues, then aucune surface ne propose `run adopt`; elle propose de déplacer ou supprimer le chemin, puis de relancer le plan.

#### US-003: Borner import et adoption aux preuves legacy

**Description:** As an utilisateur legacy, I want transférer uniquement les assets que mon lock v3 prouve so that mes autres fichiers homonymes restent étrangers au receipt Arthur.

**Priority:** P0
**Size:** L (5 pts)
**Dependencies:** Blocked by US-001, US-002

**Acceptance Criteria:**

- [ ] Given un lock v3 contenant un `source_id` catalogue et un skill strictement conforme, when `adopt` ou la réconciliation inspecte l'état, then ce skill devient `adoptable`.
- [ ] Given une activation Claude strictement dérivable d'un `source_id` legacy vérifié, when sa cible exacte est validée, then elle peut partager la preuve de cette entrée sans créer une preuve pour un autre asset.
- [ ] Given un agent, un support ou un skill absent du lock, when son contenu correspond au catalogue, then il reste `conflict` et n'entre ni dans `adoption_entries` ni dans le receipt projeté.
- [ ] Given un lock mixte avec entrées catalogue et hors catalogue, when l'adoption commit, then seules les entrées catalogue vérifiées sont transférées et le lock résiduel conserve toutes les autres.
- [ ] Given une installation sans receipt avec un lock qui ne prouve qu'un sous-ensemble des chemins présents, when la transition d'import est construite, then aucun autre chemin observé n'est injecté dans une baseline synthétique.
- [ ] Given une entrée legacy dont contenu, type, mode, cible ou identité de lock est invalide, when l'adoption est planifiée, then elle produit un diagnostic avec `source_id` et destination, puis réalise zéro claim partielle.
- [ ] Given le lock absent ou vide, when `adopt --dry-run` s'exécute, then la liste des candidats vérifiés est vide, le résultat est `noop` code 0 et chaque collision homonyme reste non possédée.

---

### EP-002: Décision unique et convergence observable

Construire une décision de cycle de vie canonique et la faire consommer sans divergence par chaque commande, renderer et opération de control plane.

**Definition of Done:** une même requête produit un seul graphe de décision, le receipt projeté en fait partie, et toutes les surfaces affichent les mêmes actions, blockers, provenances et remédiations.

**Amendements constatés pendant l'implémentation (v1.1):**

- Un chemin désiré absent est classé `create` même lorsqu'un receipt le prouvait: rien n'est écrasé, la précondition de l'opération reste `Missing` et la claim vaut `created_in_transaction`. `drifted` est réservé aux chemins présents qui diffèrent à la fois de leur preuve et du catalogue (US-002 AC1 prévaut sur une lecture élargie de US-002 AC5).
- La transaction impose exactement une opération de receipt. Une requête qui archive et réécrit un lock v3 vérifié planifie donc explicitement `WriteReceipt` (`ReceiptChangeReason::LegacyLockRewrite`), sans quoi la mutation du lock resterait sans enregistrement transactionnel. US-006 AC3 continue de s'appliquer aux requêtes sans mutation externe.

#### US-004: Construire une LifecycleDecision canonique

**Description:** As a développeur du CLI, I want calculer une décision complète depuis une requête typée so that aucune commande ne sélectionne une transition concurrente après le rendu.

**Priority:** P0
**Size:** L (5 pts)
**Dependencies:** Blocked by US-001, US-003

**Acceptance Criteria:**

- [ ] Given une requête `reconcile`, when le builder s'exécute, then il consomme providers, catalogue désiré, état observé, receipt et inspection legacy avant de produire une décision.
- [ ] Given une décision réussie, when elle est inspectée, then elle contient les entrées d'assets, claims de provenance, vérifications attendues, diagnostics, applicabilité, résumé et receipt projeté.
- [ ] Given une requête `adopt`, when la décision est calculée, then elle ne sélectionne que les candidats `verified_legacy`; les chemins non prouvés restent hors du plan d'adoption et ne peuvent produire ni blocker ni `MissingLegacyEntry` pour cette commande.
- [ ] Given deux calculs sur des inputs byte-identiques, when les décisions sont normalisées hors identifiant de transaction, then elles sont byte-identiques.
- [ ] Given une erreur d'inspection, un receipt futur ou un lock legacy non supporté, when le builder termine, then il retourne une décision bloquée complète plutôt qu'une transition alternative partielle.
- [ ] Given un renderer ou l'exécuteur, when il reçoit la décision, then il n'accède pas au filesystem pour recalculer une classification métier.

#### US-005: Router plan, dry-run, UI et apply vers la même décision

**Description:** As an utilisateur, I want que chaque surface présente la décision réellement appliquée so that confirmer l'UI n'introduise aucun changement non annoncé.

**Priority:** P0
**Size:** L (5 pts)
**Dependencies:** Blocked by US-004

**Acceptance Criteria:**

- [ ] Given les mêmes providers et le même état, when `plan`, `install --dry-run` et `update --dry-run` sont exécutés, then leurs opérations et diagnostics normalisés sont identiques.
- [ ] Given une installation interactive, when l'écran Review s'affiche, then son résumé et ses groupes Changes dérivent de la même `LifecycleDecision`.
- [ ] Given un apply confirmé, when l'exécuteur démarre, then il reçoit la décision présentée et n'appelle aucun second planner ou sélecteur de transition.
- [ ] Given les renderers Ratatui, plain, humain non interactif et JSON, when une décision est rendue, then chaque asset conserve le même statut, la même destination, la même base d'ownership et le même diagnostic.
- [ ] Given une décision bloquée, when l'UI atteint Review, then Apply est désactivé et la remédiation ne mentionne `adopt` que si au moins un candidat vérifié existe.
- [ ] Given une divergence introduite entre assessment, plan final ou opérations, when les tests de projection s'exécutent, then ils échouent avant toute transaction.

#### US-006: Représenter et committer la convergence du receipt

**Description:** As an utilisateur installé, I want que les changements de receipt soient planifiés explicitement so that l'état déclaré converge même lorsque le contenu disque est déjà correct.

**Priority:** P0
**Size:** M (3 pts)
**Dependencies:** Blocked by US-004

**Acceptance Criteria:**

- [ ] Given un chemin prouvé par le receipt dont le noeud courant égale le catalogue désiré, when l'ancien hash de receipt diffère, then l'asset est `noop` et une opération receipt-only met à jour sa preuve.
- [ ] Given tous les assets `noop` mais une version catalogue, un provider, un hash ou une référence de receipt obsolète, when la décision est calculée, then `already_current` vaut false et `WriteReceipt` apparaît dans le plan.
- [ ] Given un receipt projeté sémantiquement identique au receipt courant, when la décision est calculée, then aucune opération `WriteReceipt` n'est créée et le résultat peut être `noop`.
- [ ] Given la comparaison sémantique des receipts, when elle s'exécute, then les identifiants transactionnels et timestamps alloués au commit ne provoquent pas seuls une mutation.
- [ ] Given une transaction receipt-only réussie, when la même commande est relancée, then elle retourne `noop`, code 0 et ne modifie aucun mtime.
- [ ] Given le receipt projeté impossible à sérialiser ou à écrire avec les permissions requises, when la transaction s'exécute, then aucun succès n'est annoncé et le rollback ou `RECOVERY_REQUIRED` suit le contrat existant.

#### US-007: Exposer provenance et diagnostics actionnables

**Description:** As an utilisateur ou automate, I want connaître la preuve et le chemin derrière chaque blocker so that je puisse résoudre la collision sans inspection manuelle du code.

**Priority:** P1
**Size:** M (3 pts)
**Dependencies:** Blocked by US-004

**Acceptance Criteria:**

- [ ] Given une opération ou entrée de plan JSON, when elle est sérialisée, then elle expose un `ownership_basis` stable parmi `receipt`, `verified_legacy`, `created_in_transaction` et `none`.
- [ ] Given une entrée legacy invalide ou une collision sans preuve, when le diagnostic est projeté, then il conserve le code, un champ `source_id` optionnel, la destination via `path_utf8` ou `path_bytes_hex` et une remédiation exécutable.
- [ ] Given un chemin identique sans preuve, when la sortie humaine, plain ou Ratatui est rendue, then elle le nomme `matching unmanaged` ou son équivalent localisé et n'utilise pas le label `Adopt`.
- [ ] Given une décision contenant à la fois des candidats legacy et des collisions étrangères, when le résumé est rendu, then les deux comptes sont séparés et leur somme correspond aux entrées détaillées.
- [ ] Given le JSON schema v1, when les fixtures historiques sont désérialisées par le nouveau code, then elles restent acceptées; le nouveau champ est additif et aucun champ existant ne change de type.
- [ ] Given un path Unix non UTF-8, when un diagnostic est produit, then il utilise `path_bytes_hex`, ne fabrique pas une chaîne lossy et bloque la mutation conformément au contrat v1.

---

### EP-003: Commit sûr et preuve de non-régression

Fermer la fenêtre entre confirmation et receipt commit, puis prouver le comportement sur les états mixtes qui ont révélé le défaut.

**Definition of Done:** aucune claim périmée ne peut entrer dans un receipt; les fixtures couvrent provenance partielle, receipt périmé, renderers et changements concurrents sur les cinq surfaces de décision.

**Amendements constatés pendant l'implémentation (v1.2):**

- Toute précondition périmée détectée avant la première mutation produit désormais `stale_lifecycle_decision`, statut `blocked` et code 3, y compris le chemin qui retournait auparavant un échec de transaction en code 5 lors de la construction des opérations. FR-17 prévaut sur la classification historique de `StalePlan`: aucune écriture n'a eu lieu, donc le résultat est un conflit et non une transaction échouée.
- La revalidation ne se limite pas aux assets nouvellement revendiqués: toute entrée du receipt projeté que la transaction n'écrit pas est revérifiée, y compris une entrée déjà possédée. Une transaction dont le receipt republierait la preuve d'un asset dérivé est donc refusée avec son chemin exact. Cela concrétise FR-16 et la mitigation du risque 1.
- La revalidation compare toujours le contenu recalculé, jamais un couple mtime/taille: une modification survenue dans la même granularité d'horodatage resterait invisible autrement ([racy Git](https://git-scm.com/docs/racy-git)).

#### US-008: Revalider chaque claim avant le receipt commit

**Description:** As an utilisateur, I want que le CLI revalide les assets qu'il s'apprête à revendiquer so that une modification concurrente ne devienne pas silencieusement possédée.

**Priority:** P0
**Size:** L (5 pts)
**Dependencies:** Blocked by US-003, US-004, US-006

**Acceptance Criteria:**

- [ ] Given une décision applicable, when la transaction prend le verrou Arthur, then elle revalide avant la première mutation les fingerprints de lock, receipt et assets utilisés par chaque claim.
- [ ] Given un asset adopté ou receipt-only qui ne reçoit aucune écriture de contenu, when le commit du receipt approche, then son type, hash, mode, cible et identité disponible sont revalidés immédiatement avant `WriteReceipt`.
- [ ] Given un asset ou lock changé avant la première mutation, when la précondition est contrôlée, then la commande retourne `blocked`, code 3, avec `stale_lifecycle_decision` et réalise zéro mutation.
- [ ] Given un asset changé après le début d'une transaction mais avant receipt commit, when la seconde revalidation échoue, then la transaction retourne le code 5, rollback toutes ses mutations et n'écrit aucun receipt revendiquant cet asset.
- [ ] Given le rollback incapable de restaurer une mutation antérieure, when la transaction termine, then elle conserve journal et backups, retourne `RECOVERY_REQUIRED` et ne déclare pas l'installation saine.
- [ ] Given un asset créé ou remplacé par la transaction elle-même, when il est revalidé, then le fingerprint attendu provient de l'opération appliquée et non du snapshot pré-transaction.
- [ ] Given un writer externe qui ne respecte pas le verrou Arthur, when il modifie répétitivement l'asset, then aucune tentative ne contourne la vérification en convertissant la claim en ownership par présence.

#### US-009: Prouver par contrats la parité et les états mixtes

**Description:** As a mainteneur, I want une matrice de tests ciblée sur les preuves et projections so that une future extension du catalogue ne réintroduise pas la divergence.

**Priority:** P0
**Size:** L (5 pts)
**Dependencies:** Blocked by US-005, US-006, US-007, US-008

**Acceptance Criteria:**

- [ ] Given une fixture avec receipt ancien, un skill legacy prouvé, un skill homonyme identique non prouvé et quatre hashes receipt périmés, when toutes les commandes read-only s'exécutent, then leurs décisions normalisées respectent exactement les règles EP-001 et EP-002.
- [ ] Given la fixture mixte, when `adopt --dry-run` s'exécute, then seul le skill prouvé est candidat et aucun `MissingLegacyEntry` n'est émis pour le skill non prouvé.
- [ ] Given une installation sans receipt et un lock legacy partiel, when import, dry-run puis apply s'enchaînent, then le receipt final ne contient que les assets créés ou prouvés.
- [ ] Given un receipt dont le hash est ancien mais dont le disque égale le nouveau catalogue, when update s'applique, then seule la convergence nécessaire est commitée et le second run est no-op.
- [ ] Given un changement d'asset injecté entre Review et receipt commit, when apply s'exécute, then le scénario prouve zéro faux ownership et le rollback attendu.
- [ ] Given une décision commune, when JSON, plain, TUI et opérations executor sont projetés, then une assertion de parité couvre statuts, destinations, provenances, diagnostics et comptes.
- [ ] Given les tests d'adoption atomique, de rollback, de signaux et de récupération existants, when la suite complète s'exécute, then aucun contrat antérieur de sûreté ne régresse.
- [ ] Given une fixture qui pointe vers un HOME live ou sort du `TempDir`, when le harness démarre, then le test échoue avant toute mutation.

## Functional Requirements

- FR-01: Une requête de cycle de vie doit produire une seule `LifecycleDecision` avant rendu ou mutation.
- FR-02: La décision doit intégrer catalogue désiré, état observé, receipt, inspection legacy, providers, diagnostics, applicabilité et receipt projeté.
- FR-03: L'ownership antérieur doit provenir exclusivement d'un receipt valide.
- FR-04: L'adoptabilité doit provenir exclusivement d'une entrée Vercel Skills v3 vérifiée et d'assets associés strictement conformes.
- FR-05: Un chemin identique sans preuve doit être classé `conflict` avec le code `matching_unmanaged_without_proof`.
- FR-06: Un chemin divergent sans preuve doit rester `conflict` et ne doit jamais être converti en update.
- FR-07: Un import sans receipt ne doit pas fabriquer une baseline d'ownership depuis les chemins observés.
- FR-08: Les opérations d'import doivent conserver la provenance asset par asset jusqu'au receipt projeté.
- FR-09: `plan`, `install --dry-run` et `update --dry-run` doivent produire la même décision normalisée pour une requête reconcile équivalente.
- FR-10: Ratatui, plain, humain non interactif, JSON et l'exécuteur doivent projeter la même décision sans classifier à nouveau les assets.
- FR-11: Apply doit exécuter la décision confirmée et ne doit pas recalculer une transition métier.
- FR-12: Les opérations de receipt-only doivent apparaître dans le plan et empêcher un faux `already_current`.
- FR-13: La comparaison de receipt doit ignorer uniquement les valeurs volatiles allouées au commit et comparer toutes les valeurs sémantiques.
- FR-14: Une seconde application après convergence doit produire zéro mutation.
- FR-15: Chaque claim d'ownership doit porter une base parmi `receipt`, `verified_legacy` ou `created_in_transaction`.
- FR-16: Chaque claim doit être revalidée avant mutation et toute claim sans écriture de contenu doit l'être à nouveau avant receipt commit.
- FR-17: Une précondition périmée avant mutation doit produire `stale_lifecycle_decision`, code 3 et zéro écriture.
- FR-18: Une précondition périmée pendant transaction doit déclencher rollback et code 5, ou `RECOVERY_REQUIRED` si la compensation ne termine pas.
- FR-19: Aucun diagnostic ne doit recommander `adopt` lorsqu'aucun candidat `verified_legacy` n'existe.
- FR-20: Une requête `adopt` doit limiter son plan aux candidats `verified_legacy`; les autres collisions catalogue restent hors scope et sont évaluées par une requête reconcile.
- FR-21: Les diagnostics de provenance doivent conserver code, destination lossless, champ `source_id` optionnel et remédiation.
- FR-22: Le JSON v1 doit ajouter `ownership_basis` aux entrées de plan et `source_id` aux diagnostics sans retirer ou renommer un champ existant.
- FR-23: Les codes de sortie et statuts publics existants doivent rester inchangés.
- FR-24: `plan` et tous les dry-runs doivent rester read-only, y compris pendant inspection legacy et projection du receipt.
- FR-25: Aucune commande de ce chantier ne doit accéder au réseau ou exécuter un contenu du catalogue.
- FR-26: Le README et les aides de commande doivent définir `adopt` comme transfert Vercel v3 vérifié et indiquer déplacement ou suppression pour un matching unmanaged sans preuve.

## Non-Functional Requirements

- **Performance:** sur la référence Linux x86_64 de 2 vCPU, 7 GB RAM et SSD du PRD parent, `plan --json` doit rester sous 250 ms au P95 sur 30 processus froids pour le catalogue courant.
- **Determinism:** 30 calculs d'une même décision doivent produire 100% de sorties byte-identiques après normalisation des identifiants transactionnels et timestamps.
- **Security:** 100% des records d'ownership écrits dans un receipt doivent référencer une preuve revalidée; 0 chemin `ownership_basis=none` ne peut apparaître comme possédé.
- **Reliability:** 100% des injections de changement avant commit doivent atteindre zéro mutation, rollback complet ou `RECOVERY_REQUIRED`; 0 succès partiel silencieux est accepté.
- **Compatibility:** 100% des fixtures JSON v1 historiques doivent rester désérialisables et aucun code de sortie public ne doit changer.
- **Parity:** 100% des fixtures de cycle de vie doivent produire les mêmes statuts, destinations et diagnostics sur plan, dry-run, TUI, plain, JSON et executor.
- **Privacy:** les commandes concernées doivent produire 0 accès réseau, 0 lecture de credentials provider et 0 télémétrie.
- **Accessibility:** 100% des blockers et remédiations doivent être présents en texte dans Ratatui et plain; aucune information critique ne peut dépendre uniquement de la couleur.
- **Scalability:** la décision doit traiter 500 skills et 50 agents sous 750 ms au P95 et sous 64 MB de RSS sur la référence du PRD parent.
- **Maintainability:** le workspace doit conserver au moins 90% de couverture de régions LLVM, 0 warning Clippy, 0 bloc unsafe et 0 unwrap/expect de production non motivé.
- **Portability:** 100% des contrats filesystem critiques doivent passer sur Linux, macOS et Windows dans la matrice de release existante.

## Edge Cases & Error States

| # | Scenario | Trigger | Expected Behavior | User Message |
|---|----------|---------|-------------------|--------------|
| 1 | Installation vierge avec collision identique | Aucun receipt, aucun lock, chemin homonyme identique | `conflict`, aucune claim, aucune mutation | "Matching unmanaged asset has no ownership proof. Move or remove it, then retry." |
| 2 | Lock legacy partiel | Un skill prouvé et un skill identique absent du lock | Seul le premier est adoptable; le second reste conflictuel | "Only verified legacy entries can be adopted." |
| 3 | Lock legacy vide | `.skill-lock.json` v3 sans entrée catalogue | Zéro candidat adoptable, chemins présents non possédés | "No verified catalog entries were found in the legacy lock." |
| 4 | Entrée legacy invalide | Hash, mode, type ou cible divergent | Adoption bloquée avec source et destination exactes | "Legacy entry does not match the bundled catalog: <path>." |
| 5 | Agent identique non prouvé | Agent présent mais absent du modèle legacy | `conflict`, jamais ignoré ou adopté silencieusement | "Matching unmanaged agent has no ownership proof: <path>." |
| 6 | Receipt périmé, disque égal au désiré | Ancien hash receipt, bytes courants égaux au catalogue | No-op asset plus opération receipt-only | "Installation metadata will be reconciled." |
| 7 | Receipt périmé, disque divergent | Bytes courants différents du receipt et du catalogue | `drifted`, transaction bloquée | "Managed asset has local changes and was preserved: <path>." |
| 8 | Receipt sémantiquement courant | Seuls transaction ID ou timestamp projetés diffèrent | Aucun WriteReceipt, résultat no-op | "Configuration is already current." |
| 9 | Lock modifié après Review | Writer externe change le lock avant apply | Décision périmée, code 3, zéro mutation | "Lifecycle decision became stale. Review a new plan." |
| 10 | Asset modifié avant première mutation | Writer externe change hash, mode, type ou cible | Décision périmée, code 3, zéro mutation | "Asset changed after planning: <path>." |
| 11 | Asset modifié pendant transaction | Changement après une mutation, avant receipt commit | Rollback, code 5 ou recovery required | "Asset changed during commit. Previous changes were rolled back." |
| 12 | Receipt futur ou corrompu | Schéma inconnu ou JSON invalide | Décision bloquée, aucune fallback par présence | "Installation receipt is unreadable or newer than this CLI." |
| 13 | Chemin non UTF-8 | Destination avec bytes invalides | Mutation bloquée, `path_bytes_hex` lossless | "Non-UTF-8 paths are not supported in v1." |
| 14 | Deux commandes Arthur concurrentes | Verrou déjà détenu | Seconde commande refusée sous 250 ms | "Another Arthur Workflow transaction is running." |
| 15 | Dry-run ou refus de confirmation | Utilisateur n'applique pas la décision | Zéro lock persistant, staging, receipt ou mtime modifié | "No changes were applied." |
| 16 | Échec de rollback | Précondition inverse perdue | `RECOVERY_REQUIRED`, backups conservés | "Recovery is required before another mutation." |

## Risks & Mitigations

| # | Risk | Probability | Impact | Mitigation |
|---|------|------------|--------|------------|
| 1 | Une installation legacy auparavant acceptée devient bloquée car sa preuve était implicite | Med | High | Fixture de compatibilité, diagnostic par chemin et procédure move/remove documentée |
| 2 | Une preuve legacy est étendue à des assets que le lock ne représente pas | Med | High | Mapping fermé par `source_id`, validation asset par asset et tests agents/support négatifs |
| 3 | La décision unique devient un objet agrégateur difficile à maintenir | Med | Med | Réutiliser les types existants, garder le builder pur et séparer scan, décision, projection et exécution |
| 4 | La comparaison sémantique du receipt ignore un champ significatif | Low | High | Allowlist explicite des seuls champs volatils et test de mutation pour chaque autre champ |
| 5 | La seconde revalidation détecte une course après des mutations | Med | High | Rollback existant, fault injection avant chaque borne et receipt commit en dernier |
| 6 | Les champs JSON additifs cassent un consommateur strict non conforme | Low | Med | Fixtures historiques, note de changelog et absence de suppression ou renommage |
| 7 | L'inspection legacy intégrée ralentit chaque plan | Low | Med | Benchmark P95, lecture unique par décision et aucune duplication par renderer |
| 8 | Les tests process restent lents et masquent la boucle de feedback | Med | Med | Contrats purs pour la majorité de la matrice, scénarios process réservés aux frontières transactionnelles |

## Non-Goals

Explicit boundaries, what this version does NOT include:

- Ajouter une adoption générique fondée uniquement sur une confirmation utilisateur.
- Ajouter un flag `--force` pour revendiquer, écraser ou supprimer un asset non possédé.
- Persister un plan sur disque ou permettre son application dans un autre process.
- Introduire un nouveau statut public de plan lorsque `conflict` avec une raison typée suffit.
- Changer le format du lock Vercel Skills v3 ou prendre ownership de ses entrées hors catalogue.
- Transformer `arthur-skills` en gestionnaire de packages multi-catalogues ou multi-sources.
- Ajouter un backend, une télémétrie, une synchronisation distante ou un accès réseau.
- Ajouter une crate, Tokio ou un runtime JavaScript.
- Reconcevoir le journal, le staging, le rollback, les signaux ou la récupération au-delà des vérifications de claim requises.
- Modifier les contenus du catalogue, les agents, leurs prompts, leurs modèles ou leurs permissions.
- Réécrire le PRD parent ou son status historique. Le présent PRD documente l'amendement.
- Changer les racines provider, les stratégies symlink/copie ou la visibilité implicite Codex.

## Files NOT to Modify

- `/home/arthur/.agents/**` - installation personnelle live; toutes les fixtures utilisent un HOME temporaire.
- `/home/arthur/.claude/**` - configuration Claude live, hors surface de test.
- `/home/arthur/.codex/**` - configuration Codex live, hors surface de test.
- `skills/**` - contenu du catalogue sans rapport avec le moteur de cycle de vie.
- `agents/**` - agents publiés et evals sans rapport avec ce correctif.
- `shared/**` - supports runtime dont les bytes ne changent pas dans ce chantier.
- `tasks/prd-arthur-workflow-installer.md` - PRD parent conservé comme historique.
- `tasks/prd-arthur-workflow-installer-status.json` - état du chantier parent déjà terminé.
- `.github/workflows/release.yml` - pipeline de publication non concerné.
- `.git/**` - métadonnées Git.

## Technical Considerations

Frame as questions for engineering input, not mandates:

- **Decision boundary:** où placer `LifecycleDecision` sans créer un module transversal artificiel? Recommandation: commencer dans `lifecycle.rs` près des transitions existantes, puis extraire uniquement si les dépendances de scan, projection et exécution ne restent pas orientées.
- **Provenance model:** faut-il persister la provenance détaillée dans le receipt ou la garder dans la décision? Recommandation: réutiliser les états et `source_id` déjà persistés, conserver les fingerprints de vérification dans la décision et n'ajouter un champ receipt que si un invariant courant est impossible à prouver autrement.
- **Planner classification:** comment représenter un matching unmanaged sans statut public supplémentaire? Recommandation: `PlanKind::Conflict` avec un reason code fermé `matching_unmanaged_without_proof`.
- **Legacy mapping:** comment une entrée skill prouve-t-elle une activation Claude? Recommandation: mapping fermé depuis le même `source_id`, validé par destination, type et cible exacte; agents et supports restent hors preuve sauf représentation explicite du schéma legacy.
- **Control plane:** comment empêcher `already_current` de masquer un receipt-only update? Recommandation: comparer le receipt projeté avant le raccourci no-op et représenter `WriteReceipt` dans les opérations de la décision.
- **Volatile fields:** quels champs exclure de l'égalité sémantique? Recommandation: allowlist limitée à l'identifiant et aux timestamps alloués pour le commit; chaque autre champ reçoit un test qui force `WriteReceipt`.
- **Apply contract:** comment revalider les claims sans dupliquer les préconditions? Recommandation: réutiliser `ExpectedNode` et les snapshots transactionnels, avec une liste de claims non mutantes contrôlée avant le receipt commit.
- **JSON compatibility:** comment exposer `ownership_basis` et `source_id` en v1? Recommandation: champs additifs, valeurs fermées ou nullables, golden fixtures anciennes et nouvelles, aucun bump de major.
- **Migration:** faut-il migrer les receipts existants au démarrage? Recommandation: aucune mutation au démarrage; la prochaine décision applicable projette le receipt courant et le commit transactionnel réalise la convergence.
- **Testing:** quelle part doit rester en tests process? Recommandation: contrats purs pour classification, provenance et parité; `TempDir` process pour verrou, confirmation, revalidation, rollback et codes de sortie.

## Success Metrics

| Metric | Baseline (current) | Target | Timeframe | How Measured |
|--------|-------------------|--------|-----------|-------------|
| Parité des décisions | 3 décisions différentes sur l'état reproduit | 100% d'égalité normalisée sur toutes les fixtures | Month-1 et chaque release | Tests contractuels plan/dry-run/UI/executor |
| Faux candidats adoptables | 7 assets `write-agent-guidance` proposés sans lock | 0 asset adoptable sans preuve legacy | Month-1 | Fixture mixed provenance et sorties JSON |
| Claims sans preuve | Import capable de synthétiser l'ownership depuis les chemins observés | 0 record receipt sans preuve autorisée | Month-1 et Month-6 | Assertions sur chaque receipt de la matrice |
| Convergence receipt-only | Sortie anticipée possible avant `WriteReceipt` | 100% des receipts convergent en une transaction, second run no-op | Month-1 | Fixtures stale receipt et snapshots mtime |
| Diagnostics actionnables | `MissingLegacyEntry` peut perdre chemin et source | 100% des blockers avec code, path et remédiation | Month-1 | Golden JSON, plain et Ratatui |
| Résistance aux courses | Claims no-op non revalidées avant receipt commit | 100% des injections détectées sans faux ownership | Month-1 et chaque release | Fault injection transactionnelle |
| Performance plan | Contrat parent de 250 ms P95, non mesuré pendant l'incident | P95 inférieur à 250 ms sur 30 runs | Month-1 | Benchmark CI de la référence parent |
| Régressions de sûreté | Suites adoption et planner séparées | 0 régression sur rollback, recover, lock et adoption atomique | Month-6 | Quality gates et matrice release |

## Open Questions

- Faut-il un jour permettre l'adoption volontaire d'un matching unmanaged sans lock externe? Owner: Arthur. Deadline: revue Month-6 à partir des demandes réelles. Dépendance: nouveau modèle de preuve, hors scope de ce PRD.
- Un plan persisté et signé devient-il nécessaire pour une exécution différée ou distante? Owner: Arthur. Deadline: revue après apparition d'un cas d'usage non interactif démontré. Dépendance: futur schéma de plan, hors scope de ce PRD.
[/PRD]
