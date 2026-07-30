use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::adoption::LegacyImportPlan;
use crate::catalog::{AssetKind, Catalog, Provider as CatalogProvider};
use crate::engine::{EngineError, plan_desired_state_with_removal_policy};
use crate::plan::{
    DesiredAsset, DesiredPayload, Diagnostic, LegacyOwnership, LegacyProofScope, MutationKind,
    OwnershipClaim, OwnershipProof, PLAN_SCHEMA_VERSION, Plan, PlanAction, PlanEntry, PlanReason,
    PlanSummary, PlannedInverse, PlannedMutation, Precondition, RemovalPolicy, inspect_snapshot,
};
use crate::provider::{ProviderId, ProviderRegistry, ResolvedProvider, ResolvedRoots};
use crate::receipt::{OwnedAsset, OwnedAssetKind, Receipt, ReceiptError, RetainedUnmanagedAsset};
use crate::transaction::{PathKind, snapshot_path};

const DIRECTORY_MODE: u32 = 0o755;
const LEGACY_IMPORT_ENTRY_LIMIT: usize = 100_000;
const RECEIPT_SOURCE_ID: &str = "receipt:v1";
const RECEIPT_MODE: u32 = 0o600;
/// Stable identifier of the receipt mutation. The executor writes the receipt
/// last, so its identifier sorts after every filesystem operation.
pub const WRITE_RECEIPT_OPERATION_ID: &str = "zzzzzzzz-write-receipt";

/// Diagnostic codes a blocked decision can carry.
pub const UNSAFE_CONTAINER: &str = "unsafe_container";
pub const RECEIPT_UNREADABLE: &str = "receipt_unreadable";
pub const LEGACY_LOCK_UNSUPPORTED: &str = "legacy_lock_unsupported";

/// The typed request every lifecycle surface starts from.
///
/// One request plus one observed state produces exactly one decision, so no
/// command can select a competing transition after the plan was rendered.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "request")]
pub enum LifecycleRequest {
    /// Converge the desired catalog without claiming any unproven destination.
    Reconcile {
        providers: Vec<ProviderId>,
    },
    /// Transfer a verified Vercel Skills v3 installation that has no receipt.
    Import {
        providers: Vec<ProviderId>,
    },
    /// Adopt only the destinations a verified v3 lock entry proves.
    Adopt {
        providers: Vec<ProviderId>,
    },
    UninstallProvider(ProviderId),
    UninstallAll,
}

impl LifecycleRequest {
    #[must_use]
    pub fn providers(&self) -> Vec<ProviderId> {
        match self {
            Self::Reconcile { providers } | Self::Import { providers } => providers.clone(),
            Self::Adopt { providers } => providers.clone(),
            Self::UninstallProvider(_) | Self::UninstallAll => Vec::new(),
        }
    }

    const fn is_uninstall(&self) -> bool {
        matches!(self, Self::UninstallProvider(_) | Self::UninstallAll)
    }
}

/// What the shared read-only inspection of the legacy lock proved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LegacyEvidence<'a> {
    /// No lock, or no catalog entry inside it.
    Absent,
    /// A verified v3 lock: every entry it names can prove its own destinations.
    Verified(&'a LegacyImportPlan),
    /// The lock exists but this CLI cannot verify it.
    Unsupported { detail: String },
}

impl<'a> LegacyEvidence<'a> {
    #[must_use]
    pub const fn verified(&self) -> Option<&'a LegacyImportPlan> {
        match self {
            Self::Verified(plan) => Some(plan),
            Self::Absent | Self::Unsupported { .. } => None,
        }
    }
}

impl<'a> From<Option<&'a LegacyImportPlan>> for LegacyEvidence<'a> {
    fn from(plan: Option<&'a LegacyImportPlan>) -> Self {
        plan.map_or(Self::Absent, Self::Verified)
    }
}

/// Why the projected receipt differs from the recorded one.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptChangeReason {
    NoCurrentReceipt,
    CliVersion,
    CatalogVersion,
    Roots,
    Providers,
    Assets,
    RetainedUnmanaged,
    State,
    FilesystemMutations,
    /// A verified legacy lock is archived and rewritten by this transaction, and
    /// every transaction commits exactly one receipt.
    LegacyLockRewrite,
}

/// Control-plane convergence of the decision.
///
/// A receipt whose only difference is the identifier allocated at commit time is
/// semantically current, so it must not produce a mutation.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ReceiptChange {
    pub required: bool,
    pub reasons: Vec<ReceiptChangeReason>,
}

/// The single artefact every surface consumes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LifecycleDecision {
    pub request: LifecycleRequest,
    pub selected_providers: Vec<ProviderId>,
    pub plan: Plan,
    /// Receipt projected by this decision; only proven destinations appear in it.
    pub receipt: Receipt,
    pub receipt_change: ReceiptChange,
    pub notices: Vec<LifecycleNotice>,
    pub summary: PlanSummary,
}

impl LifecycleDecision {
    #[must_use]
    pub const fn applicable(&self) -> bool {
        self.plan.applicable
    }

    /// True when nothing on disk and nothing in the receipt has to change.
    #[must_use]
    pub fn is_current(&self) -> bool {
        self.plan.operations.is_empty()
            && !self.receipt_change.required
            && self
                .plan
                .entries
                .iter()
                .all(|entry| entry.action == PlanAction::Noop)
    }

    /// The verified legacy candidates, the only paths `adopt` may transfer.
    pub fn adoption_candidates(&self) -> impl Iterator<Item = &PlanEntry> {
        self.plan
            .entries
            .iter()
            .filter(|entry| entry.action == PlanAction::Adoptable)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleNoticeCode {
    ClaudeRestartRequired,
    CodexUsesImplicitSkills,
    CodexMayDiscoverCanonicalSkills,
    CodexIntegrationRemovedSkillsRemainVisible,
}

impl LifecycleNoticeCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeRestartRequired => "claude_restart_required",
            Self::CodexUsesImplicitSkills => "codex_uses_implicit_skills",
            Self::CodexMayDiscoverCanonicalSkills => "codex_may_discover_canonical_skills",
            Self::CodexIntegrationRemovedSkillsRemainVisible => {
                "codex_integration_removed_skills_remain_visible"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LifecycleNotice {
    pub code: LifecycleNoticeCode,
    pub message: String,
}

#[derive(Debug)]
pub enum LifecycleError {
    EmptyProviderSelection,
    MissingProviderRoot(ProviderId),
    InvalidCatalogPath(String),
    MissingEmbeddedFile(String),
    UnsafeContainer { path: PathBuf, detail: String },
    Engine(EngineError),
    Receipt(ReceiptError),
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProviderSelection => {
                formatter.write_str("install requires at least one provider")
            }
            Self::MissingProviderRoot(provider) => {
                write!(formatter, "resolved roots do not include {provider}")
            }
            Self::InvalidCatalogPath(path) => {
                write!(
                    formatter,
                    "catalog path is not valid for installation: {path}"
                )
            }
            Self::MissingEmbeddedFile(path) => {
                write!(formatter, "catalog bytes are missing for {path}")
            }
            Self::UnsafeContainer { path, detail } => {
                write!(
                    formatter,
                    "unsafe shared container {}: {detail}",
                    path.display()
                )
            }
            Self::Engine(error) => error.fmt(formatter),
            Self::Receipt(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for LifecycleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Engine(error) => Some(error),
            Self::Receipt(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EngineError> for LifecycleError {
    fn from(error: EngineError) -> Self {
        Self::Engine(error)
    }
}

impl From<ReceiptError> for LifecycleError {
    fn from(error: ReceiptError) -> Self {
        Self::Receipt(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ManagedDesired {
    asset: DesiredAsset,
    references: Vec<ProviderId>,
}

/// Maps every verified v3 lock entry to the destinations it can prove.
///
/// The v3 lock records one entry per skill name and carries no per-file, agent
/// or support record, so a proven entry covers exactly its canonical skill
/// directory and the Claude activation strictly derivable from the same name.
/// Every other observed path stays unproven.
fn legacy_ownership(roots: &ResolvedRoots, legacy: Option<&LegacyImportPlan>) -> LegacyOwnership {
    let Some(legacy) = legacy else {
        return LegacyOwnership::default();
    };
    let claude_skills = roots
        .provider(ProviderId::Claude)
        .and_then(|provider| provider.skills.as_deref());
    let mut scopes = Vec::new();
    for name in legacy
        .managed_skill_names
        .iter()
        .chain(legacy.obsolete_skill_names.iter())
    {
        for root in std::iter::once(roots.canonical_skills.join(name))
            .chain(claude_skills.map(|skills| skills.join(name)))
        {
            scopes.push(LegacyProofScope {
                root,
                source_id: name.clone(),
                lock_sha256: legacy.original_hash.clone(),
            });
        }
    }
    LegacyOwnership::new(scopes)
}

/// Builds the one decision every surface consumes.
///
/// An unverifiable receipt, an unsupported legacy lock or a failed filesystem
/// inspection produce a complete blocked decision instead of an alternative
/// transition, so no command can fall back to a partial plan.
pub fn decide(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    current: Option<&Receipt>,
    request: &LifecycleRequest,
    legacy: &LegacyEvidence<'_>,
) -> Result<LifecycleDecision, LifecycleError> {
    if let LegacyEvidence::Unsupported { detail } = legacy {
        return Ok(blocked_decision(
            catalog,
            roots,
            request,
            Diagnostic::path_error(
                LEGACY_LOCK_UNSUPPORTED,
                detail.clone(),
                &roots.legacy_lock_path,
            ),
        ));
    }
    match build_decision(catalog, roots, current, request, legacy.verified()) {
        Ok(decision) => Ok(decision),
        Err(LifecycleError::UnsafeContainer { path, detail }) => Ok(blocked_decision(
            catalog,
            roots,
            request,
            Diagnostic::path_error(UNSAFE_CONTAINER, detail, &path),
        )),
        Err(
            LifecycleError::Receipt(error) | LifecycleError::Engine(EngineError::Receipt(error)),
        ) => Ok(blocked_decision(
            catalog,
            roots,
            request,
            Diagnostic::path_error(RECEIPT_UNREADABLE, error.to_string(), &roots.receipt_path),
        )),
        Err(other) => Err(other),
    }
}

/// A complete decision that can never be applied and claims nothing.
fn blocked_decision(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    request: &LifecycleRequest,
    diagnostic: Diagnostic,
) -> LifecycleDecision {
    let plan = Plan {
        schema_version: PLAN_SCHEMA_VERSION,
        applicable: false,
        entries: Vec::new(),
        operations: Vec::new(),
        diagnostics: vec![diagnostic],
    };
    let summary = plan.summary();
    LifecycleDecision {
        request: request.clone(),
        selected_providers: request.providers(),
        receipt: Receipt::new(
            env!("CARGO_PKG_VERSION"),
            &catalog.manifest().catalog_sha256,
            roots,
        ),
        plan,
        receipt_change: ReceiptChange::default(),
        notices: Vec::new(),
        summary,
    }
}

fn build_decision(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    current: Option<&Receipt>,
    request: &LifecycleRequest,
    legacy: Option<&LegacyImportPlan>,
) -> Result<LifecycleDecision, LifecycleError> {
    if let Some(receipt) = current {
        receipt.validate()?;
        receipt.validate_roots(roots)?;
    }
    let current_providers = managed_providers(current);
    let selected_providers = selected_after(request, &current_providers)?;
    let required_roots = current_providers
        .iter()
        .chain(selected_providers.iter())
        .copied()
        .collect::<BTreeSet<_>>();
    for provider in required_roots {
        if roots.provider(provider).is_none() {
            return Err(LifecycleError::MissingProviderRoot(provider));
        }
    }

    let managed = build_desired(catalog, roots, current, &selected_providers)?;
    let proofs = legacy_ownership(roots, legacy);
    // Only an import may seed a baseline from verified legacy proofs. Every
    // other request keeps the recorded receipt as its sole prior ownership.
    let baseline = match request {
        LifecycleRequest::Import { .. } => Some(import_baseline(
            catalog,
            roots,
            &selected_providers,
            &managed,
            &proofs,
            legacy,
        )?),
        _ => current.cloned(),
    };
    let desired = managed
        .values()
        .map(|entry| entry.asset.clone())
        .collect::<Vec<_>>();
    let removal_policy = if request.is_uninstall() {
        RemovalPolicy::RetainUnmanaged
    } else {
        RemovalPolicy::BlockOnDrift
    };
    let mut plan = plan_desired_state_with_removal_policy(
        roots,
        baseline.as_ref(),
        &desired,
        &proofs,
        removal_policy,
    )?;
    let mut receipt = build_receipt(
        catalog,
        roots,
        baseline.as_ref(),
        &selected_providers,
        &managed,
        &plan,
    )?;
    let notices = lifecycle_notices(
        request,
        &current_providers,
        &selected_providers,
        &plan,
        roots,
    );

    if matches!(request, LifecycleRequest::Adopt { .. }) {
        // `adopt` owns the verified candidates only. Unproven collisions are
        // evaluated by a reconcile request and must not block this command.
        // The transfer itself claims each verified entry after the lock is
        // reverified, so this decision projects the recorded receipt unchanged.
        plan = adoption_scoped_plan(&plan);
        receipt = current.cloned().unwrap_or_else(|| {
            Receipt::new(
                env!("CARGO_PKG_VERSION"),
                &catalog.manifest().catalog_sha256,
                roots,
            )
        });
    }

    let receipt_change = receipt_change(current, &receipt, &plan, request, legacy.is_some());
    if receipt_change.required {
        push_receipt_convergence(&mut plan, roots)?;
    }
    let summary = plan.summary();
    Ok(LifecycleDecision {
        request: request.clone(),
        selected_providers,
        plan,
        receipt,
        receipt_change,
        notices,
        summary,
    })
}

/// Seeds an import baseline from verified legacy proofs only.
fn import_baseline(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    selected_providers: &[ProviderId],
    managed: &BTreeMap<PathBuf, ManagedDesired>,
    proofs: &LegacyOwnership,
    legacy: Option<&LegacyImportPlan>,
) -> Result<Receipt, LifecycleError> {
    let mut baseline = Receipt::new(
        env!("CARGO_PKG_VERSION"),
        &catalog.manifest().catalog_sha256,
        roots,
    );
    for provider in &mut baseline.providers {
        provider.managed_integration = selected_providers.contains(&provider.provider);
    }
    for entry in managed.values() {
        if proofs.proof_for(&entry.asset.destination).is_none() {
            continue;
        }
        if let Some(asset) = observed_owned_asset(
            &entry.asset.source_id,
            &entry.asset.destination,
            &entry.references,
        )? {
            baseline.assets.push(asset);
        }
    }
    if let Some(legacy) = legacy {
        for name in &legacy.obsolete_skill_names {
            collect_legacy_skill(
                &roots.canonical_skills.join(name),
                name,
                selected_providers,
                &mut baseline.assets,
            )?;
        }
    }
    baseline
        .assets
        .sort_by(|left, right| left.destination.cmp(&right.destination));
    baseline.validate()?;
    Ok(baseline)
}

/// Restricts a decision to the verified legacy candidates `adopt` owns.
fn adoption_scoped_plan(source: &Plan) -> Plan {
    let entries = source
        .entries
        .iter()
        .filter(|entry| entry.action == PlanAction::Adoptable)
        .cloned()
        .collect::<Vec<_>>();
    let destinations = entries
        .iter()
        .map(|entry| entry.destination.as_path())
        .collect::<BTreeSet<_>>();
    Plan {
        schema_version: source.schema_version,
        applicable: true,
        diagnostics: source
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic
                    .path_utf8
                    .as_deref()
                    .is_some_and(|path| destinations.contains(Path::new(path)))
            })
            .cloned()
            .collect(),
        entries,
        operations: Vec::new(),
    }
}

/// Compares the projected receipt with the recorded one.
///
/// Only the identifier allocated at commit time is volatile, so every other
/// difference is a real convergence the plan must show.
fn receipt_change(
    current: Option<&Receipt>,
    projected: &Receipt,
    plan: &Plan,
    request: &LifecycleRequest,
    legacy_lock_rewrite: bool,
) -> ReceiptChange {
    if matches!(request, LifecycleRequest::Adopt { .. }) || !plan.applicable {
        return ReceiptChange::default();
    }
    let mut reasons = match current {
        None => vec![ReceiptChangeReason::NoCurrentReceipt],
        Some(current) => semantic_receipt_differences(current, projected),
    };
    if plan.has_mutations() {
        reasons.push(ReceiptChangeReason::FilesystemMutations);
    }
    if legacy_lock_rewrite {
        reasons.push(ReceiptChangeReason::LegacyLockRewrite);
    }
    reasons.sort_unstable();
    reasons.dedup();
    ReceiptChange {
        required: !reasons.is_empty(),
        reasons,
    }
}

fn semantic_receipt_differences(
    current: &Receipt,
    projected: &Receipt,
) -> Vec<ReceiptChangeReason> {
    let mut reasons = Vec::new();
    if current.cli_version != projected.cli_version {
        reasons.push(ReceiptChangeReason::CliVersion);
    }
    if current.catalog_sha256 != projected.catalog_sha256 {
        reasons.push(ReceiptChangeReason::CatalogVersion);
    }
    if current.roots != projected.roots {
        reasons.push(ReceiptChangeReason::Roots);
    }
    if current.providers != projected.providers {
        reasons.push(ReceiptChangeReason::Providers);
    }
    if !owned_assets_equal(current, projected) {
        reasons.push(ReceiptChangeReason::Assets);
    }
    if !retained_equal(current, projected) {
        reasons.push(ReceiptChangeReason::RetainedUnmanaged);
    }
    if current.state != projected.state {
        reasons.push(ReceiptChangeReason::State);
    }
    reasons
}

fn owned_assets_equal(current: &Receipt, projected: &Receipt) -> bool {
    let left = current
        .assets
        .iter()
        .map(|asset| (asset.destination.as_path(), asset))
        .collect::<BTreeMap<_, _>>();
    let right = projected
        .assets
        .iter()
        .map(|asset| (asset.destination.as_path(), asset))
        .collect::<BTreeMap<_, _>>();
    left == right
}

fn retained_equal(current: &Receipt, projected: &Receipt) -> bool {
    let left = current
        .retained_unmanaged
        .iter()
        .map(|entry| (entry.destination.as_path(), entry))
        .collect::<BTreeMap<_, _>>();
    let right = projected
        .retained_unmanaged
        .iter()
        .map(|entry| (entry.destination.as_path(), entry))
        .collect::<BTreeMap<_, _>>();
    left == right
}

/// Represents the receipt commit in the same plan as the filesystem changes.
///
/// The executor commits the receipt only when the plan carries this marker, so a
/// semantically current receipt is never rewritten.
pub fn push_receipt_convergence(
    plan: &mut Plan,
    roots: &ResolvedRoots,
) -> Result<(), LifecycleError> {
    let snapshot =
        inspect_snapshot(&roots.receipt_path).map_err(|error| LifecycleError::UnsafeContainer {
            path: roots.receipt_path.clone(),
            detail: error.to_string(),
        })?;
    let (precondition, inverse, ownership) = match snapshot {
        None => (
            Precondition::Missing,
            PlannedInverse::RemoveCreated,
            OwnershipProof::UnownedDestination,
        ),
        Some(snapshot) => {
            let ownership = OwnershipProof::Receipt {
                source_id: RECEIPT_SOURCE_ID.to_owned(),
                sha256: snapshot.sha256.clone(),
            };
            (
                Precondition::Matches { snapshot },
                PlannedInverse::RestoreBackup,
                ownership,
            )
        }
    };
    plan.entries.push(PlanEntry::new(
        PlanAction::WriteReceipt,
        RECEIPT_SOURCE_ID,
        roots.receipt_path.clone(),
        crate::plan::Owner::ArthurWorkflow,
        PlanReason::ReceiptConvergence,
        OwnershipClaim::CreatedInTransaction {
            operation_id: WRITE_RECEIPT_OPERATION_ID.to_owned(),
        },
    ));
    plan.operations.push(PlannedMutation {
        id: WRITE_RECEIPT_OPERATION_ID.to_owned(),
        kind: MutationKind::WriteReceipt,
        root: roots.canonical.lexical.clone(),
        destination: roots.receipt_path.clone(),
        precondition,
        inverse,
        ownership,
        content_sha256: None,
        mode: Some(RECEIPT_MODE),
        link_target: None,
        payload: None,
    });
    Ok(())
}

fn observed_owned_asset(
    source_id: &str,
    destination: &Path,
    references: &[ProviderId],
) -> Result<Option<OwnedAsset>, LifecycleError> {
    let snapshot = snapshot_path(destination).map_err(|error| LifecycleError::UnsafeContainer {
        path: destination.to_path_buf(),
        detail: error.to_string(),
    })?;
    let (kind, hash, mode, link_target) = match snapshot.kind {
        PathKind::Absent => return Ok(None),
        PathKind::File => (OwnedAssetKind::File, snapshot.sha256, snapshot.mode, None),
        PathKind::Directory => (OwnedAssetKind::Directory, None, snapshot.mode, None),
        PathKind::Symlink => (OwnedAssetKind::Symlink, None, None, snapshot.link_target),
    };
    Ok(Some(OwnedAsset {
        source_id: source_id.to_owned(),
        destination: destination.to_path_buf(),
        kind,
        hash,
        mode,
        link_target,
        references: references.to_vec(),
    }))
}

fn collect_legacy_skill(
    root: &Path,
    name: &str,
    references: &[ProviderId],
    assets: &mut Vec<OwnedAsset>,
) -> Result<(), LifecycleError> {
    let root_metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(LifecycleError::UnsafeContainer {
                path: root.to_path_buf(),
                detail: error.to_string(),
            });
        }
    };
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        if let Some(asset) =
            observed_owned_asset(&legacy_source_id(root, root, name)?, root, references)?
        {
            assets.push(asset);
        }
        return Ok(());
    }
    let canonical_root =
        fs::canonicalize(root).map_err(|error| LifecycleError::UnsafeContainer {
            path: root.to_path_buf(),
            detail: error.to_string(),
        })?;
    let mut pending = vec![root.to_path_buf()];
    let mut imported = Vec::new();
    while let Some(path) = pending.pop() {
        if imported.len() >= LEGACY_IMPORT_ENTRY_LIMIT {
            return Err(LifecycleError::UnsafeContainer {
                path,
                detail: format!(
                    "legacy skill exceeds the {LEGACY_IMPORT_ENTRY_LIMIT} entry safety limit"
                ),
            });
        }
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| LifecycleError::UnsafeContainer {
                path: path.to_path_buf(),
                detail: error.to_string(),
            })?;
        if !metadata.file_type().is_symlink() {
            let canonical =
                fs::canonicalize(&path).map_err(|error| LifecycleError::UnsafeContainer {
                    path: path.to_path_buf(),
                    detail: error.to_string(),
                })?;
            if !canonical.starts_with(&canonical_root) {
                return Err(LifecycleError::UnsafeContainer {
                    path,
                    detail: "legacy skill traversal escaped its recorded directory".to_owned(),
                });
            }
        }
        let Some(asset) =
            observed_owned_asset(&legacy_source_id(root, &path, name)?, &path, references)?
        else {
            continue;
        };
        if asset.kind == OwnedAssetKind::Directory {
            let entries = fs::read_dir(&path).map_err(|error| LifecycleError::UnsafeContainer {
                path: path.to_path_buf(),
                detail: error.to_string(),
            })?;
            for entry in entries {
                let entry = entry.map_err(|error| LifecycleError::UnsafeContainer {
                    path: path.to_path_buf(),
                    detail: error.to_string(),
                })?;
                pending.push(entry.path());
            }
        }
        imported.push(asset);
    }
    assets.extend(imported);
    Ok(())
}

fn legacy_source_id(root: &Path, path: &Path, name: &str) -> Result<String, LifecycleError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| LifecycleError::InvalidCatalogPath(error.to_string()))?;
    if relative.as_os_str().is_empty() {
        return Ok(format!("legacy:skills/{name}"));
    }
    let relative = relative
        .to_str()
        .ok_or_else(|| LifecycleError::InvalidCatalogPath(path.display().to_string()))?;
    Ok(format!("legacy:skills/{name}/{relative}"))
}

fn managed_providers(receipt: Option<&Receipt>) -> Vec<ProviderId> {
    receipt.map_or_else(Vec::new, |receipt| {
        receipt
            .providers
            .iter()
            .filter(|provider| provider.managed_integration)
            .map(|provider| provider.provider)
            .collect()
    })
}

fn selected_after(
    request: &LifecycleRequest,
    current: &[ProviderId],
) -> Result<Vec<ProviderId>, LifecycleError> {
    let selected = match request {
        LifecycleRequest::Reconcile { providers }
        | LifecycleRequest::Import { providers }
        | LifecycleRequest::Adopt { providers } => {
            if providers.is_empty() {
                return Err(LifecycleError::EmptyProviderSelection);
            }
            providers.iter().copied().collect::<BTreeSet<_>>()
        }
        LifecycleRequest::UninstallProvider(removed) => current
            .iter()
            .copied()
            .filter(|provider| provider != removed)
            .collect(),
        LifecycleRequest::UninstallAll => BTreeSet::new(),
    };
    Ok(selected.into_iter().collect())
}

fn build_desired(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    current: Option<&Receipt>,
    selected: &[ProviderId],
) -> Result<BTreeMap<PathBuf, ManagedDesired>, LifecycleError> {
    let mut desired = BTreeMap::new();
    if selected.is_empty() {
        return Ok(desired);
    }

    maybe_insert_container(
        &mut desired,
        current,
        "container:canonical-skills",
        &roots.canonical_skills,
        selected,
    )?;
    insert_canonical_skills(catalog, roots, selected, &mut desired)?;

    if selected.contains(&ProviderId::Claude) {
        let provider = required_provider(roots, ProviderId::Claude)?;
        insert_provider_containers(&mut desired, current, provider, ProviderId::Claude)?;
        insert_claude_activations(catalog, roots, provider, &mut desired)?;
        insert_provider_files(catalog, provider, ProviderId::Claude, &mut desired)?;
        insert_claude_support(catalog, current, provider, &mut desired)?;
    }
    if selected.contains(&ProviderId::Codex) {
        let provider = required_provider(roots, ProviderId::Codex)?;
        insert_provider_containers(&mut desired, current, provider, ProviderId::Codex)?;
        insert_provider_files(catalog, provider, ProviderId::Codex, &mut desired)?;
    }
    Ok(desired)
}

fn insert_canonical_skills(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    references: &[ProviderId],
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
) -> Result<(), LifecycleError> {
    for asset in catalog
        .manifest()
        .assets
        .iter()
        .filter(|asset| asset.kind == AssetKind::Skill)
    {
        let asset_path = Path::new(&asset.relative_path);
        let skill_relative = strip_catalog_prefix(asset_path, Path::new("skills"))?;
        insert_directory(
            desired,
            format!("directory:{}", asset.relative_path),
            roots.canonical_skills.join(skill_relative),
            references,
        )?;

        for record in &asset.files {
            let record_path = Path::new(&record.relative_path);
            let mut parent = record_path.parent();
            while let Some(directory) = parent.filter(|path| path.starts_with(asset_path)) {
                let relative = strip_catalog_prefix(directory, Path::new("skills"))?;
                insert_directory(
                    desired,
                    format!("directory:{}", directory.display()),
                    roots.canonical_skills.join(relative),
                    references,
                )?;
                if directory == asset_path {
                    break;
                }
                parent = directory.parent();
            }
            let relative = strip_catalog_prefix(record_path, Path::new("skills"))?;
            insert_catalog_file(
                catalog,
                desired,
                &record.relative_path,
                roots.canonical_skills.join(relative),
                record.mode,
                references,
            )?;
        }
    }
    Ok(())
}

fn insert_provider_containers(
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
    current: Option<&Receipt>,
    provider: &ResolvedProvider,
    provider_id: ProviderId,
) -> Result<(), LifecycleError> {
    let references = [provider_id];
    maybe_insert_container(
        desired,
        current,
        &format!("container:{}-root", provider_id.as_str()),
        &provider.root.lexical,
        &references,
    )?;
    if let Some(skills) = &provider.skills {
        maybe_insert_container(
            desired,
            current,
            &format!("container:{}-skills", provider_id.as_str()),
            skills,
            &references,
        )?;
    }
    maybe_insert_container(
        desired,
        current,
        &format!("container:{}-agents", provider_id.as_str()),
        &provider.agents,
        &references,
    )
}

#[cfg(unix)]
fn insert_claude_activations(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    provider: &ResolvedProvider,
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
) -> Result<(), LifecycleError> {
    let skills_root = provider
        .skills
        .as_ref()
        .ok_or(LifecycleError::MissingProviderRoot(ProviderId::Claude))?;
    for asset in catalog
        .manifest()
        .assets
        .iter()
        .filter(|asset| asset.kind == AssetKind::Skill)
    {
        let name = Path::new(&asset.relative_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| LifecycleError::InvalidCatalogPath(asset.relative_path.clone()))?;
        insert_managed(
            desired,
            DesiredAsset {
                source_id: format!("activation:claude:{name}"),
                destination: skills_root.join(name),
                payload: DesiredPayload::Symlink {
                    target: PathBuf::from(format!("../../.agents/skills/{name}")),
                    canonical_target: roots.canonical_skills.join(name),
                },
            },
            &[ProviderId::Claude],
        )?;
    }
    Ok(())
}

#[cfg(windows)]
fn insert_claude_activations(
    catalog: &Catalog,
    _roots: &ResolvedRoots,
    provider: &ResolvedProvider,
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
) -> Result<(), LifecycleError> {
    let skills_root = provider
        .skills
        .as_ref()
        .ok_or(LifecycleError::MissingProviderRoot(ProviderId::Claude))?;
    for asset in catalog
        .manifest()
        .assets
        .iter()
        .filter(|asset| asset.kind == AssetKind::Skill)
    {
        let asset_path = Path::new(&asset.relative_path);
        let skill_relative = strip_catalog_prefix(asset_path, Path::new("skills"))?;
        insert_directory(
            desired,
            format!("activation:claude:directory:{}", asset.relative_path),
            skills_root.join(skill_relative),
            &[ProviderId::Claude],
        )?;
        for record in &asset.files {
            let record_path = Path::new(&record.relative_path);
            let mut parent = record_path.parent();
            while let Some(directory) = parent.filter(|path| path.starts_with(asset_path)) {
                let relative = strip_catalog_prefix(directory, Path::new("skills"))?;
                insert_directory(
                    desired,
                    format!("activation:claude:directory:{}", directory.display()),
                    skills_root.join(relative),
                    &[ProviderId::Claude],
                )?;
                if directory == asset_path {
                    break;
                }
                parent = directory.parent();
            }
            let relative = strip_catalog_prefix(record_path, Path::new("skills"))?;
            insert_catalog_file(
                catalog,
                desired,
                &record.relative_path,
                skills_root.join(relative),
                record.mode,
                &[ProviderId::Claude],
            )?;
        }
    }
    Ok(())
}

fn insert_provider_files(
    catalog: &Catalog,
    provider: &ResolvedProvider,
    provider_id: ProviderId,
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
) -> Result<(), LifecycleError> {
    let (catalog_provider, prefix) = match provider_id {
        ProviderId::Claude => (CatalogProvider::Claude, Path::new("agents/claude")),
        ProviderId::Codex => (CatalogProvider::Codex, Path::new("agents/codex")),
    };
    for asset in
        catalog.manifest().assets.iter().filter(|asset| {
            asset.kind == AssetKind::Agent && asset.provider == Some(catalog_provider)
        })
    {
        for record in &asset.files {
            let relative = strip_catalog_prefix(Path::new(&record.relative_path), prefix)?;
            insert_catalog_file(
                catalog,
                desired,
                &record.relative_path,
                provider.agents.join(relative),
                record.mode,
                &[provider_id],
            )?;
        }
    }
    Ok(())
}

fn insert_claude_support(
    catalog: &Catalog,
    current: Option<&Receipt>,
    provider: &ResolvedProvider,
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
) -> Result<(), LifecycleError> {
    let skills_root = provider
        .skills
        .as_ref()
        .ok_or(LifecycleError::MissingProviderRoot(ProviderId::Claude))?;
    let shared_root = skills_root.join("_shared");
    maybe_insert_container(
        desired,
        current,
        "container:claude-shared",
        &shared_root,
        &[ProviderId::Claude],
    )?;
    let prefix = Path::new("shared/claude/skills");
    for asset in catalog
        .manifest()
        .assets
        .iter()
        .filter(|asset| asset.kind == AssetKind::Support)
    {
        for record in &asset.files {
            let relative = strip_catalog_prefix(Path::new(&record.relative_path), prefix)?;
            insert_catalog_file(
                catalog,
                desired,
                &record.relative_path,
                skills_root.join(relative),
                record.mode,
                &[ProviderId::Claude],
            )?;
        }
    }
    Ok(())
}

fn maybe_insert_container(
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
    current: Option<&Receipt>,
    source_id: &str,
    destination: &Path,
    references: &[ProviderId],
) -> Result<(), LifecycleError> {
    if current
        .and_then(|receipt| receipt.owned_asset(destination))
        .is_some()
    {
        return insert_directory(
            desired,
            source_id.to_owned(),
            destination.to_path_buf(),
            references,
        );
    }
    match fs::symlink_metadata(destination) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            Ok(())
        }
        Ok(_) => Err(LifecycleError::UnsafeContainer {
            path: destination.to_path_buf(),
            detail: "expected a real directory".to_owned(),
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => insert_directory(
            desired,
            source_id.to_owned(),
            destination.to_path_buf(),
            references,
        ),
        Err(error) => Err(LifecycleError::UnsafeContainer {
            path: destination.to_path_buf(),
            detail: error.to_string(),
        }),
    }
}

fn insert_directory(
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
    source_id: String,
    destination: PathBuf,
    references: &[ProviderId],
) -> Result<(), LifecycleError> {
    if let Some(existing) = desired.get_mut(&destination) {
        if existing.asset.payload
            != (DesiredPayload::Directory {
                mode: DIRECTORY_MODE,
            })
        {
            return Err(LifecycleError::InvalidCatalogPath(format!(
                "directory collides with another asset at {}",
                destination.display()
            )));
        }
        existing.references.extend_from_slice(references);
        existing.references.sort_unstable();
        existing.references.dedup();
        return Ok(());
    }
    insert_managed(
        desired,
        DesiredAsset {
            source_id,
            destination,
            payload: DesiredPayload::Directory {
                mode: DIRECTORY_MODE,
            },
        },
        references,
    )
}

fn insert_catalog_file(
    catalog: &Catalog,
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
    source_id: &str,
    destination: PathBuf,
    mode: u32,
    references: &[ProviderId],
) -> Result<(), LifecycleError> {
    let embedded = catalog
        .embedded_file(source_id)
        .ok_or_else(|| LifecycleError::MissingEmbeddedFile(source_id.to_owned()))?;
    insert_managed(
        desired,
        DesiredAsset {
            source_id: source_id.to_owned(),
            destination,
            payload: DesiredPayload::File {
                bytes: embedded.bytes.to_vec(),
                mode,
            },
        },
        references,
    )
}

fn insert_managed(
    desired: &mut BTreeMap<PathBuf, ManagedDesired>,
    asset: DesiredAsset,
    references: &[ProviderId],
) -> Result<(), LifecycleError> {
    let destination = asset.destination.clone();
    let mut references = references.to_vec();
    references.sort_unstable();
    references.dedup();
    if desired
        .insert(
            asset.destination.clone(),
            ManagedDesired { asset, references },
        )
        .is_some()
    {
        return Err(LifecycleError::InvalidCatalogPath(format!(
            "duplicate destination {}",
            destination.display()
        )));
    }
    Ok(())
}

fn strip_catalog_prefix<'a>(path: &'a Path, prefix: &Path) -> Result<&'a Path, LifecycleError> {
    let relative = path
        .strip_prefix(prefix)
        .map_err(|_| LifecycleError::InvalidCatalogPath(path.display().to_string()))?;
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(LifecycleError::InvalidCatalogPath(
            path.display().to_string(),
        ));
    }
    Ok(relative)
}

fn required_provider(
    roots: &ResolvedRoots,
    provider: ProviderId,
) -> Result<&ResolvedProvider, LifecycleError> {
    roots
        .provider(provider)
        .ok_or(LifecycleError::MissingProviderRoot(provider))
}

fn build_receipt(
    catalog: &Catalog,
    roots: &ResolvedRoots,
    current: Option<&Receipt>,
    selected: &[ProviderId],
    managed: &BTreeMap<PathBuf, ManagedDesired>,
    plan: &Plan,
) -> Result<Receipt, LifecycleError> {
    let mut receipt = Receipt::new(
        env!("CARGO_PKG_VERSION"),
        &catalog.manifest().catalog_sha256,
        roots,
    );
    for provider in &mut receipt.providers {
        provider.managed_integration = selected.contains(&provider.provider);
        provider.implicit_skill_visibility = ProviderRegistry::get(provider.provider)
            .capabilities
            .implicit_skill_visibility;
        if provider.root.is_none() {
            provider.root = current.and_then(|receipt| {
                receipt
                    .providers
                    .iter()
                    .find(|prior| prior.provider == provider.provider)
                    .and_then(|prior| prior.root.clone())
            });
        }
    }
    // A projected receipt may only claim destinations the plan proved. A
    // conflicting or unproven path is never written as owned, whatever its
    // name or bytes.
    // An adoptable destination is proven by the lock but not yet transferred,
    // so only `adopt` may claim it. Every other provable entry is claimable.
    let provable = plan
        .entries
        .iter()
        .filter(|entry| {
            entry.ownership_basis().is_provable() && entry.action != PlanAction::Adoptable
        })
        .map(|entry| entry.destination.as_path())
        .collect::<BTreeSet<_>>();
    receipt.assets = managed
        .values()
        .filter(|entry| provable.contains(entry.asset.destination.as_path()))
        .map(owned_asset)
        .collect();

    let mut retained = current
        .into_iter()
        .flat_map(|receipt| receipt.retained_unmanaged.iter().cloned())
        .filter(|entry| !managed.contains_key(&entry.destination))
        .map(|entry| (entry.destination.clone(), entry))
        .collect::<BTreeMap<_, _>>();
    for entry in plan
        .entries
        .iter()
        .filter(|entry| entry.action == PlanAction::RetainedUnmanaged)
    {
        retained.insert(
            entry.destination.clone(),
            RetainedUnmanagedAsset {
                source_id: entry.source.clone(),
                destination: entry.destination.clone(),
                reason: entry.reason.message().to_owned(),
            },
        );
    }
    receipt.retained_unmanaged = retained.into_values().collect();
    receipt.validate()?;
    Ok(receipt)
}

fn owned_asset(entry: &ManagedDesired) -> OwnedAsset {
    let expected = entry.asset.payload.expected();
    let kind = match expected.kind {
        crate::plan::NodeKind::Directory => OwnedAssetKind::Directory,
        crate::plan::NodeKind::File => OwnedAssetKind::File,
        crate::plan::NodeKind::Symlink => OwnedAssetKind::Symlink,
    };
    OwnedAsset {
        source_id: entry.asset.source_id.clone(),
        destination: entry.asset.destination.clone(),
        kind,
        hash: expected.sha256,
        mode: expected.mode,
        link_target: expected.link_target,
        references: entry.references.clone(),
    }
}

fn lifecycle_notices(
    request: &LifecycleRequest,
    current: &[ProviderId],
    selected: &[ProviderId],
    plan: &Plan,
    roots: &ResolvedRoots,
) -> Vec<LifecycleNotice> {
    let mut notices = Vec::new();
    if selected.contains(&ProviderId::Claude)
        && let Some(skills) = roots
            .provider(ProviderId::Claude)
            .and_then(|provider| provider.skills.as_ref())
        && plan
            .entries
            .iter()
            .any(|entry| entry.destination == *skills && entry.action == PlanAction::Create)
    {
        notices.push(LifecycleNotice {
            code: LifecycleNoticeCode::ClaudeRestartRequired,
            message: format!(
                "Restart Claude Code after creating {} if it was already running; new top-level skill directories are watched only after startup.",
                skills.display()
            ),
        });
    }
    if selected.contains(&ProviderId::Codex) {
        notices.push(LifecycleNotice {
            code: LifecycleNoticeCode::CodexUsesImplicitSkills,
            message: "Codex reads the canonical skills directly; only its agents are managed as an integration."
                .to_owned(),
        });
    } else if !selected.is_empty() {
        notices.push(LifecycleNotice {
            code: LifecycleNoticeCode::CodexMayDiscoverCanonicalSkills,
            message: "A Codex installation can discover the canonical skills while another provider keeps them installed."
                .to_owned(),
        });
    }
    if matches!(
        request,
        LifecycleRequest::UninstallProvider(ProviderId::Codex)
    ) && current.contains(&ProviderId::Codex)
        && !selected.is_empty()
    {
        notices.push(LifecycleNotice {
            code: LifecycleNoticeCode::CodexIntegrationRemovedSkillsRemainVisible,
            message: "Codex agents are removed, but canonical skills remain discoverable while another provider references them."
                .to_owned(),
        });
    }
    notices.sort_by_key(|notice| notice.code);
    notices
}

#[cfg(test)]
#[path = "lifecycle/tests.rs"]
mod tests;
