//! EP-003 contracts: no stale claim can enter a receipt, and the mixed states
//! that revealed the defect are pinned on every decision surface.
//!
//! The prior adoption, rollback, signal and recovery contracts keep their own
//! suites: this file adds evidence and never relaxes one of them.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use arthur_skills::adoption::{self, LegacyImportPlan};
use arthur_skills::app::{App, Review};
use arthur_skills::catalog::{AssetKind, Catalog};
use arthur_skills::lifecycle::{
    LegacyEvidence, LifecycleDecision, LifecycleRequest, ReceiptChangeReason, decide,
};
use arthur_skills::operations::{TransactionPlan, operations_for_plan};
use arthur_skills::output::{Envelope, write_human, write_json};
use arthur_skills::plain;
use arthur_skills::plan::{
    MATCHING_UNMANAGED_WITHOUT_PROOF, OwnershipBasis, PlanAction, PlanReason, action_key,
};
use arthur_skills::provider::{ProviderId, ResolvedRoots, resolve_roots_from};
use arthur_skills::receipt::Receipt;
use arthur_skills::transaction::{
    FailureInjector, MutationPoint, MutationPrimitive, SignalFlags, TransactionEngine,
    TransactionError, TransactionOutcome,
};
use arthur_skills::ui::render;
use arthur_skills::workflow::assess_decision;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn Error>>;

/// Root names of a live installation, which no fixture may ever reach.
const LIVE_INSTALL_ROOTS: [&str; 3] = [".agents", ".claude", ".codex"];

/// Refuses a fixture path that leaves its sandbox or reaches a live root.
fn guard_path(path: &Path, sandbox: &Path, canonical_sandbox: &Path) -> Result<(), String> {
    if !path.starts_with(sandbox) && !path.starts_with(canonical_sandbox) {
        return Err(format!(
            "fixture path escapes the temporary HOME: {}",
            path.display()
        ));
    }
    if let Some(live) = std::env::var_os("HOME").map(PathBuf::from) {
        for name in LIVE_INSTALL_ROOTS {
            if path.starts_with(live.join(name)) {
                return Err(format!(
                    "fixture path points at a live installation: {}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

fn guard_roots(roots: &ResolvedRoots, sandbox: &Path) -> Result<(), String> {
    let canonical = sandbox.canonicalize().map_err(|error| error.to_string())?;
    for root in roots.allowed_top_level_roots() {
        guard_path(&root.lexical, sandbox, &canonical)?;
        guard_path(&root.real, sandbox, &canonical)?;
    }
    guard_path(&roots.state_directory, sandbox, &canonical)?;
    guard_path(&roots.receipt_path, sandbox, &canonical)?;
    Ok(())
}

/// Resolves the fixture roots and refuses any path outside the sandbox.
fn roots(home: &TempDir, providers: &[ProviderId]) -> Result<ResolvedRoots, Box<dyn Error>> {
    let roots = resolve_roots_from(Some(home.path().as_os_str()), None, providers)?;
    guard_roots(&roots, home.path())?;
    Ok(roots)
}

fn reconcile(providers: &[ProviderId]) -> LifecycleRequest {
    LifecycleRequest::Reconcile {
        providers: providers.to_vec(),
    }
}

fn transaction_plan(
    roots: &ResolvedRoots,
    decision: &LifecycleDecision,
    transaction_id: &str,
) -> Result<TransactionPlan, Box<dyn Error>> {
    Ok(operations_for_plan(
        &decision.plan,
        roots,
        &decision.receipt,
        transaction_id,
    )?)
}

fn apply(
    roots: &ResolvedRoots,
    decision: &LifecycleDecision,
    transaction_id: &str,
) -> Result<TransactionOutcome, Box<dyn Error>> {
    let transaction = transaction_plan(roots, decision, transaction_id)?;
    let engine = TransactionEngine::new(roots.state_directory.clone(), SignalFlags::default());
    Ok(engine.apply(transaction_id, transaction.operations, &transaction.claims)?)
}

fn install(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    transaction_id: &str,
) -> Result<Receipt, Box<dyn Error>> {
    let decision = decide(
        catalog,
        roots,
        None,
        &reconcile(&ProviderId::ALL),
        &LegacyEvidence::Absent,
    )?;
    assert!(decision.applicable(), "{:?}", decision.plan.diagnostics);
    assert_eq!(
        apply(roots, &decision, transaction_id)?,
        TransactionOutcome::Committed
    );
    Ok(Receipt::decode(&fs::read(&roots.receipt_path)?)?)
}

fn skill_names(catalog: &Catalog) -> Vec<String> {
    catalog
        .manifest()
        .assets
        .iter()
        .filter(|asset| asset.kind == AssetKind::Skill)
        .map(|asset| asset.name.clone())
        .collect()
}

fn write_legacy_lock(roots: &ResolvedRoots, names: &[&str]) -> TestResult {
    let mut skills = serde_json::Map::new();
    for name in names {
        skills.insert(
            (*name).to_owned(),
            serde_json::json!({
                "source": "arthjean/skills",
                "sourceType": "github",
                "skillFolderHash": "0123456789012345678901234567890123456789",
                "installedAt": "2026-01-01T00:00:00.000Z",
                "updatedAt": "2026-01-01T00:00:00.000Z"
            }),
        );
    }
    fs::create_dir_all(
        roots
            .legacy_lock_path
            .parent()
            .ok_or("legacy lock has no parent")?,
    )?;
    fs::write(
        &roots.legacy_lock_path,
        serde_json::to_vec_pretty(&serde_json::json!({ "version": 3, "skills": skills }))?,
    )?;
    Ok(())
}

fn legacy_plan(
    catalog: &Catalog,
    roots: &ResolvedRoots,
) -> Result<Option<LegacyImportPlan>, Box<dyn Error>> {
    let names = skill_names(catalog).into_iter().collect::<BTreeSet<_>>();
    Ok(adoption::inspect_legacy_import(
        &roots.legacy_lock_path,
        &roots.state_directory.join("legacy-0001.json"),
        &names,
    )?)
}

/// The state that produced three incompatible decisions in 0.2.1: an older
/// receipt, one skill proven by a v3 lock, one identical skill without any
/// proof, and four receipt fingerprints left behind by a catalog change.
struct MixedFixture {
    receipt: Receipt,
    proven: PathBuf,
    unproven: PathBuf,
    stale: Vec<PathBuf>,
}

fn mixed_fixture(catalog: &Catalog, roots: &ResolvedRoots) -> Result<MixedFixture, Box<dyn Error>> {
    let committed = install(catalog, roots, "mixed-install")?;
    let names = skill_names(catalog);
    let proven_name = names.first().ok_or("the catalog has no skill")?.clone();
    let unproven_name = names.get(1).ok_or("the catalog has one skill")?.clone();
    let claude_skills = roots
        .provider(ProviderId::Claude)
        .and_then(|provider| provider.skills.clone())
        .ok_or("claude skills root is missing")?;
    let unowned = [
        roots.canonical_skills.join(&proven_name),
        roots.canonical_skills.join(&unproven_name),
        claude_skills.join(&proven_name),
        claude_skills.join(&unproven_name),
    ];

    // Both skills stay on disk byte for byte; only their receipt records go,
    // so one is proven by the lock alone and the other by nothing at all.
    let mut receipt = committed;
    receipt.assets.retain(|asset| {
        !unowned
            .iter()
            .any(|root| asset.destination.starts_with(root))
    });

    let mut stale = Vec::new();
    for asset in receipt
        .assets
        .iter_mut()
        .filter(|asset| asset.hash.is_some())
        .take(4)
    {
        asset.hash = Some("c".repeat(64));
        stale.push(asset.destination.clone());
    }
    assert_eq!(stale.len(), 4, "the fixture needs four stale fingerprints");
    receipt.validate()?;
    write_legacy_lock(roots, &[proven_name.as_str()])?;

    Ok(MixedFixture {
        receipt,
        proven: roots.canonical_skills.join(&proven_name),
        unproven: roots.canonical_skills.join(&unproven_name),
        stale,
    })
}

fn entry_action(decision: &LifecycleDecision, destination: &Path) -> Option<PlanAction> {
    decision
        .plan
        .entries
        .iter()
        .find(|entry| entry.destination == destination)
        .map(|entry| entry.action)
}

/// Normalizes a decision: everything a surface may render, never an identifier
/// allocated at commit time.
fn normalized(decision: &LifecycleDecision) -> Result<String, Box<dyn Error>> {
    Ok(serde_json::to_string(&serde_json::json!({
        "applicable": decision.plan.applicable,
        "entries": decision.plan.entries,
        "operations": decision.plan.operations,
        "diagnostics": decision.plan.diagnostics,
        "summary": decision.summary,
        "receipt_change": decision.receipt_change,
        "receipt": decision.receipt,
    }))?)
}

fn asset_mtimes(roots: &ResolvedRoots) -> Result<BTreeMap<PathBuf, SystemTime>, Box<dyn Error>> {
    let mut mtimes = BTreeMap::new();
    let mut pending = vec![roots.canonical_skills.clone()];
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                pending.push(entry.path());
            }
            mtimes.insert(entry.path(), metadata.modified()?);
        }
    }
    Ok(mtimes)
}

/// A writer that ignores the advisory transaction lock, as flock(2) permits.
struct ChangeDuringTransaction {
    primitive: MutationPrimitive,
    target: PathBuf,
    bytes: &'static [u8],
    fired: bool,
}

impl FailureInjector for ChangeDuringTransaction {
    fn after_mutation(&mut self, point: &MutationPoint) -> Result<(), String> {
        if !self.fired && point.primitive == self.primitive {
            self.fired = true;
            fs::write(&self.target, self.bytes).map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

#[test]
fn a_mixed_provenance_fixture_decides_identically_on_every_read_only_command() -> TestResult {
    let home = tempfile::tempdir()?;
    let roots = roots(&home, &ProviderId::ALL)?;
    let catalog = Catalog::load()?;
    let fixture = mixed_fixture(&catalog, &roots)?;
    let legacy = legacy_plan(&catalog, &roots)?.ok_or("the v3 lock proves nothing")?;

    // `plan`, `install --dry-run` and `update --dry-run` all converge the same
    // installation, so they resolve one request and one decision.
    let decisions = (0..3)
        .map(|_| {
            decide(
                &catalog,
                &roots,
                Some(&fixture.receipt),
                &reconcile(&ProviderId::ALL),
                &LegacyEvidence::Verified(&legacy),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let reference = normalized(&decisions[0])?;
    for decision in &decisions[1..] {
        assert_eq!(reference, normalized(decision)?);
    }
    let decision = &decisions[0];

    // EP-001: the lock proves one skill and nothing else.
    assert_eq!(
        entry_action(decision, &fixture.proven),
        Some(PlanAction::Adoptable)
    );
    for entry in &decision.plan.entries {
        if entry.destination.starts_with(&fixture.proven) {
            assert_eq!(entry.action, PlanAction::Adoptable);
            assert_eq!(entry.ownership_basis(), OwnershipBasis::VerifiedLegacy);
        }
        if entry.destination.starts_with(&fixture.unproven) {
            assert_eq!(
                entry.action,
                PlanAction::Conflict,
                "{} matches the catalog but nothing proves it",
                entry.destination.display()
            );
            assert_eq!(entry.reason, PlanReason::MatchingUnmanagedWithoutProof);
            assert_eq!(entry.ownership_basis(), OwnershipBasis::None);
        }
        assert!(
            decision.receipt.owned_asset(&entry.destination).is_none()
                || entry.ownership_basis().is_provable(),
            "{} entered the receipt without a proof",
            entry.destination.display()
        );
    }
    assert!(decision.plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == MATCHING_UNMANAGED_WITHOUT_PROOF
            && diagnostic.path_utf8.as_deref() == fixture.unproven.to_str()
    }));

    // EP-002: a stale fingerprint over identical bytes is a no-op, never drift.
    for destination in &fixture.stale {
        assert_eq!(entry_action(decision, destination), Some(PlanAction::Noop));
    }
    assert_eq!(
        decision
            .summary
            .actions
            .get(action_key(PlanAction::Drifted)),
        None
    );
    assert!(!decision.plan.applicable, "a collision blocks the plan");
    assert!(
        !decision.receipt_change.required,
        "a blocked decision converges nothing"
    );
    assert!(decision.summary.verified_legacy_candidates > 0);
    assert!(decision.summary.matching_unmanaged > 0);
    Ok(())
}

#[test]
fn adopt_selects_only_the_proven_skill_of_the_mixed_fixture() -> TestResult {
    let home = tempfile::tempdir()?;
    let roots = roots(&home, &ProviderId::ALL)?;
    let catalog = Catalog::load()?;
    let fixture = mixed_fixture(&catalog, &roots)?;
    let legacy = legacy_plan(&catalog, &roots)?.ok_or("the v3 lock proves nothing")?;

    let decision = decide(
        &catalog,
        &roots,
        Some(&fixture.receipt),
        &LifecycleRequest::Adopt {
            providers: ProviderId::ALL.to_vec(),
        },
        &LegacyEvidence::Verified(&legacy),
    )?;

    assert!(decision.applicable(), "adopt owns its verified candidates");
    let candidates = decision
        .adoption_candidates()
        .map(|entry| entry.destination.clone())
        .collect::<Vec<_>>();
    assert!(!candidates.is_empty());
    for destination in &candidates {
        assert!(
            !destination.starts_with(&fixture.unproven),
            "{} has no verified lock entry",
            destination.display()
        );
    }
    for entry in &decision.plan.entries {
        assert_eq!(entry.action, PlanAction::Adoptable);
    }
    // The unproven skill is out of scope for `adopt`: it produces no blocker,
    // so the command can never fail on a path it does not transfer.
    for diagnostic in &decision.plan.diagnostics {
        assert_ne!(
            diagnostic.path_utf8.as_deref(),
            fixture.unproven.to_str(),
            "an unproven collision must not block adopt"
        );
    }
    assert!(
        decision
            .plan
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic
                .code
                .to_ascii_lowercase()
                .contains("missinglegacy")),
        "no missing legacy entry is reported for a path adopt never claims"
    );
    Ok(())
}

#[test]
fn an_import_with_a_partial_lock_records_only_created_or_proven_assets() -> TestResult {
    let home = tempfile::tempdir()?;
    let roots = roots(&home, &ProviderId::ALL)?;
    let catalog = Catalog::load()?;
    let names = skill_names(&catalog);
    let proven_name = names.first().ok_or("the catalog has no skill")?.clone();

    // A v3 installation that proves one skill, next to a personal skill the
    // lock never names.
    let proven = roots.canonical_skills.join(&proven_name);
    fs::create_dir_all(&proven)?;
    let personal = roots.canonical_skills.join("personal-notes");
    fs::create_dir_all(&personal)?;
    let personal_file = personal.join("SKILL.md");
    fs::write(&personal_file, b"personal")?;
    write_legacy_lock(&roots, &[proven_name.as_str()])?;
    let legacy = legacy_plan(&catalog, &roots)?.ok_or("the v3 lock proves nothing")?;

    let dry_run = decide(
        &catalog,
        &roots,
        None,
        &LifecycleRequest::Import {
            providers: ProviderId::ALL.to_vec(),
        },
        &LegacyEvidence::Verified(&legacy),
    )?;
    assert!(dry_run.applicable(), "{:?}", dry_run.plan.diagnostics);
    assert!(dry_run.receipt.owned_asset(&personal_file).is_none());

    let transaction = operations_for_plan(
        &dry_run.plan,
        &roots,
        &dry_run.receipt,
        "import-partial-lock",
    )?;
    let engine = TransactionEngine::new(roots.state_directory.clone(), SignalFlags::default());
    assert_eq!(
        engine.apply(
            "import-partial-lock",
            transaction.operations,
            &transaction.claims
        )?,
        TransactionOutcome::Committed
    );

    let committed = Receipt::decode(&fs::read(&roots.receipt_path)?)?;
    assert!(
        committed.owned_asset(&personal_file).is_none(),
        "an observed path is never injected into the receipt"
    );
    assert!(committed.owned_asset(&personal).is_none());
    assert_eq!(fs::read(&personal_file)?, b"personal");
    assert!(
        committed.owned_asset(&proven.join("SKILL.md")).is_some(),
        "the proven skill is materialized and recorded"
    );
    Ok(())
}

#[test]
fn a_stale_receipt_converges_once_and_the_second_run_is_a_noop() -> TestResult {
    let home = tempfile::tempdir()?;
    let roots = roots(&home, &ProviderId::ALL)?;
    let catalog = Catalog::load()?;
    let committed = install(&catalog, &roots, "converge-install")?;
    let before = asset_mtimes(&roots)?;

    // The disk already equals the catalog; only the recorded metadata is old.
    let mut stale = committed.clone();
    stale.catalog_sha256 = "b".repeat(64);
    let decision = decide(
        &catalog,
        &roots,
        Some(&stale),
        &reconcile(&ProviderId::ALL),
        &LegacyEvidence::Absent,
    )?;
    assert!(decision.receipt_change.required);
    assert!(
        decision
            .receipt_change
            .reasons
            .contains(&ReceiptChangeReason::CatalogVersion)
    );
    assert!(
        decision
            .plan
            .entries
            .iter()
            .filter(|entry| entry.action != PlanAction::WriteReceipt)
            .all(|entry| entry.action == PlanAction::Noop)
    );

    let transaction = transaction_plan(&roots, &decision, "converge-receipt")?;
    assert_eq!(
        transaction.operations.len(),
        1,
        "only the receipt has to converge"
    );
    // Every asset the receipt claims without writing it is revalidated twice.
    let claimed = transaction
        .claims
        .claims()
        .iter()
        .map(|claim| claim.destination.clone())
        .collect::<BTreeSet<_>>();
    for asset in &decision.receipt.assets {
        assert!(
            claimed.contains(&asset.destination),
            "{} enters the receipt without a revalidated claim",
            asset.destination.display()
        );
    }

    let engine = TransactionEngine::new(roots.state_directory.clone(), SignalFlags::default());
    assert_eq!(
        engine.apply(
            "converge-receipt",
            transaction.operations,
            &transaction.claims
        )?,
        TransactionOutcome::Committed
    );
    assert_eq!(before, asset_mtimes(&roots)?, "no asset was rewritten");

    let converged = Receipt::decode(&fs::read(&roots.receipt_path)?)?;
    let again = decide(
        &catalog,
        &roots,
        Some(&converged),
        &reconcile(&ProviderId::ALL),
        &LegacyEvidence::Absent,
    )?;
    assert!(again.is_current(), "the second run is a no-op");
    assert!(!again.receipt_change.required);
    assert!(
        transaction_plan(&roots, &again, "second-run")?
            .operations
            .is_empty()
    );
    Ok(())
}

#[test]
fn a_change_injected_before_the_receipt_commit_claims_nothing() -> TestResult {
    let home = tempfile::tempdir()?;
    let roots = roots(&home, &ProviderId::ALL)?;
    let catalog = Catalog::load()?;
    let committed = install(&catalog, &roots, "race-install")?;
    let receipt_before = fs::read(&roots.receipt_path)?;
    let claimed = committed
        .assets
        .iter()
        .find(|asset| asset.hash.is_some())
        .ok_or("the receipt claims no file")?
        .destination
        .clone();

    let mut stale = committed.clone();
    stale.catalog_sha256 = "e".repeat(64);
    let decision = decide(
        &catalog,
        &roots,
        Some(&stale),
        &reconcile(&ProviderId::ALL),
        &LegacyEvidence::Absent,
    )?;
    let transaction = transaction_plan(&roots, &decision, "race-receipt")?;

    // The user confirmed this decision; a writer that ignores the lock changes
    // a claimed asset while the receipt is being staged.
    let mut injector = ChangeDuringTransaction {
        primitive: MutationPrimitive::WriteStagedFile,
        target: claimed.clone(),
        bytes: b"foreign edit",
        fired: false,
    };
    let engine = TransactionEngine::new(roots.state_directory.clone(), SignalFlags::default());
    let error = engine
        .apply_with(
            "race-receipt",
            transaction.operations,
            &transaction.claims,
            &mut injector,
        )
        .err()
        .ok_or("a racing claim must not be committed")?;
    assert!(
        matches!(&error, TransactionError::ClaimChangedDuringCommit(path) if *path == claimed),
        "unexpected error: {error}"
    );
    assert_eq!(error.exit_code(), 5);

    assert_eq!(
        fs::read(&roots.receipt_path)?,
        receipt_before,
        "the receipt never published a fingerprint it could not revalidate"
    );
    let unchanged = Receipt::decode(&fs::read(&roots.receipt_path)?)?;
    assert_eq!(
        unchanged.owned_asset(&claimed).and_then(|a| a.hash.clone()),
        committed
            .owned_asset(&claimed)
            .and_then(|asset| asset.hash.clone()),
        "no false ownership was recorded for the changed asset"
    );
    assert_eq!(fs::read(&claimed)?, b"foreign edit");
    assert_eq!(
        TransactionEngine::new(roots.state_directory.clone(), SignalFlags::default())
            .journal_state()?,
        None,
        "the compensation completed"
    );

    // The writer keeps ignoring the lock. Every later attempt sees a managed
    // path that no longer matches its proof: it is preserved as drift and never
    // reclaimed because it exists and carries the receipt's name.
    for round in 0..3 {
        fs::write(&claimed, format!("foreign edit {round}"))?;
        let retry = decide(
            &catalog,
            &roots,
            Some(&stale),
            &reconcile(&ProviderId::ALL),
            &LegacyEvidence::Absent,
        )?;
        assert!(!retry.applicable(), "round {round} must stay blocked");
        let entry = retry
            .plan
            .entries
            .iter()
            .find(|entry| entry.destination == claimed)
            .ok_or("the changed asset left the plan")?;
        assert_eq!(entry.action, PlanAction::Drifted);
        assert_eq!(entry.ownership_basis(), OwnershipBasis::Receipt);
        assert!(
            transaction_plan(&roots, &retry, "retry").is_err(),
            "a blocked decision never reaches the executor"
        );
        assert_eq!(fs::read(&roots.receipt_path)?, receipt_before);
    }
    Ok(())
}

#[test]
fn every_surface_projects_the_same_statuses_provenances_and_counts() -> TestResult {
    let home = tempfile::tempdir()?;
    let roots = roots(&home, &ProviderId::ALL)?;
    let catalog = Catalog::load()?;
    let fixture = mixed_fixture(&catalog, &roots)?;
    let legacy = legacy_plan(&catalog, &roots)?.ok_or("the v3 lock proves nothing")?;
    let decision = decide(
        &catalog,
        &roots,
        Some(&fixture.receipt),
        &reconcile(&ProviderId::ALL),
        &LegacyEvidence::Verified(&legacy),
    )?;

    // JSON keeps the status, destination, provenance and diagnostic of every
    // entry, plus the counts the summary publishes.
    let envelope = Envelope::new(Some("plan")).with_plan(&decision.plan);
    let mut json = Vec::new();
    write_json(&envelope, &mut json)?;
    let document: serde_json::Value = serde_json::from_slice(&json)?;
    let operations = document["operations"]
        .as_array()
        .ok_or("the envelope has no operations")?;
    assert_eq!(operations.len(), decision.plan.entries.len());
    for (rendered, entry) in operations.iter().zip(&decision.plan.entries) {
        assert_eq!(rendered["action"], action_key(entry.action));
        assert_eq!(
            rendered["destination_utf8"].as_str(),
            entry.destination.to_str()
        );
        assert_eq!(
            rendered["ownership_basis"],
            serde_json::to_value(entry.ownership_basis())?
        );
        assert_eq!(
            rendered["source_id"],
            serde_json::to_value(entry.ownership.source_id())?
        );
    }
    assert_eq!(
        document["summary"]["verified_legacy_candidates"],
        serde_json::to_value(decision.summary.verified_legacy_candidates)?
    );
    assert_eq!(
        document["summary"]["matching_unmanaged"],
        serde_json::to_value(decision.summary.matching_unmanaged)?
    );
    let counted = decision
        .summary
        .verified_legacy_candidates
        .checked_add(decision.summary.matching_unmanaged)
        .ok_or("collision counts overflow")?;
    assert_eq!(
        counted,
        decision
            .plan
            .entries
            .iter()
            .filter(|entry| entry.action == PlanAction::Adoptable
                || entry.reason.is_matching_unmanaged())
            .count(),
        "both collision families sum to their detailed entries"
    );

    // The human, plain and Ratatui surfaces carry the same blockers as text.
    let mut human = Vec::new();
    write_human(&envelope, &mut human)?;
    let assessment = assess_decision(Some(&fixture.receipt), &decision, 1, 0);
    let mut app = App::with_selection(catalog.skill_count(), &decision.selected_providers);
    app.set_review(Review::for_decision(&decision, &roots, assessment));
    let mut plain_output = Vec::new();
    plain::render(&app, &mut plain_output)?;
    let mut terminal = Terminal::new(TestBackend::new(120, 40))?;
    terminal.draw(|frame| render(frame, &app, false))?;
    let tui = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol().to_owned())
        .collect::<String>();
    let human = String::from_utf8(human)?;
    let plain_text = String::from_utf8(plain_output)?;

    // Human keeps every entry with its status, destination and reason.
    let proven_line = format!(
        "{} {}",
        action_key(PlanAction::Adoptable),
        fixture.proven.display()
    );
    let unproven_line = format!(
        "{} {}",
        action_key(PlanAction::Conflict),
        fixture.unproven.display()
    );
    assert!(human.contains(&proven_line), "{human}");
    assert!(human.contains(&unproven_line), "{human}");
    assert!(human.contains(PlanReason::MatchingUnmanagedWithoutProof.message()));

    // Plain and Ratatui publish the same counted provenance lines and the same
    // executable remediation, in text.
    let legacy_count = format!(
        "{} verified legacy",
        decision.summary.verified_legacy_candidates
    );
    let unmanaged_count = format!("{} matching unmanaged", decision.summary.matching_unmanaged);
    for surface in [&plain_text, &tui] {
        assert!(surface.contains(&legacy_count), "{surface}");
        assert!(surface.contains(&unmanaged_count), "{surface}");
        assert!(
            surface.contains("move or remove"),
            "the remediation must be readable as text"
        );
    }

    // An unproven collision is never labelled Adopt on any surface.
    let unproven_name = fixture
        .unproven
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("the unproven skill has no name")?;
    let labelled = plain_text
        .lines()
        .find(|line| line.trim_start().ends_with(unproven_name))
        .ok_or("the unproven skill is absent from the plain changes")?;
    assert!(labelled.trim_start().starts_with("Conflict"), "{labelled}");
    assert!(!labelled.contains("Adopt"), "{labelled}");

    // The executor consumes the same decision: every mutation it would run is
    // an entry of the plan it was given.
    let destinations = decision
        .plan
        .entries
        .iter()
        .map(|entry| entry.destination.as_path())
        .collect::<BTreeSet<_>>();
    for mutation in &decision.plan.operations {
        assert!(
            destinations.contains(mutation.destination.as_path()),
            "{} is mutated without a plan entry",
            mutation.destination.display()
        );
    }
    Ok(())
}

#[test]
fn a_fixture_outside_the_temporary_home_fails_before_any_mutation() -> TestResult {
    let home = tempfile::tempdir()?;
    let sandbox = home.path().to_path_buf();
    let canonical = sandbox.canonicalize()?;
    assert!(roots(&home, &ProviderId::ALL).is_ok());

    // A fixture resolved against another HOME never reaches this sandbox.
    let other = tempfile::tempdir()?;
    let escaped = resolve_roots_from(Some(other.path().as_os_str()), None, &ProviderId::ALL)?;
    let error = guard_roots(&escaped, &sandbox)
        .err()
        .ok_or("a foreign root must be refused")?;
    assert!(error.contains("escapes the temporary HOME"), "{error}");
    assert!(
        !escaped.state_directory.exists(),
        "the guard refuses before any state is created"
    );

    // A live installation root is refused even when it sits inside a sandbox.
    if let Some(live) = std::env::var_os("HOME").map(PathBuf::from) {
        let live_root = live.join(".claude").join("skills");
        let error = guard_path(&live_root, &live, &live)
            .err()
            .ok_or("a live root must be refused")?;
        assert!(error.contains("live installation"), "{error}");
    }
    assert!(guard_path(&sandbox.join(".agents"), &sandbox, &canonical).is_ok());
    Ok(())
}
