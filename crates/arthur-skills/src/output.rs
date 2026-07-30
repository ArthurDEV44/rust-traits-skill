use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::path::Path;

use serde::Serialize;
use serde_json::{Value, json};

use crate::plan::{
    DiagnosticSeverity as PlanSeverity, LEGACY_ENTRY_DOES_NOT_MATCH_CATALOG,
    MATCHING_UNMANAGED_WITHOUT_PROOF, Owner, OwnershipBasis, Plan, PlanAction, PlanEntry,
    PlanReason, PlanSummary, action_key,
};
use crate::platform::path_key;
use crate::provider::{ENVIRONMENT_EXIT_CODE, ProviderId};

pub const OUTPUT_SCHEMA_VERSION: u16 = 1;
pub const SUCCESS_EXIT_CODE: u8 = 0;
pub const USAGE_EXIT_CODE: u8 = 2;
pub const CONFLICT_EXIT_CODE: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStatus {
    Success,
    Noop,
    Blocked,
    Failed,
    RecoveryRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OutputDiagnostic {
    pub code: String,
    pub severity: OutputSeverity,
    pub message: String,
    /// Legacy entry identity behind the diagnostic, when one produced it.
    pub source_id: Option<String>,
    pub path_utf8: Option<String>,
    pub path_bytes_hex: Option<String>,
    pub remediation: Option<String>,
}

impl OutputDiagnostic {
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        remediation: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: OutputSeverity::Error,
            message: message.into(),
            source_id: None,
            path_utf8: None,
            path_bytes_hex: None,
            remediation,
        }
    }

    pub fn with_path(mut self, path_utf8: Option<String>, path_bytes_hex: Option<String>) -> Self {
        self.path_utf8 = path_utf8;
        self.path_bytes_hex = path_bytes_hex;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OutputOperation {
    pub action: PlanAction,
    pub source: String,
    pub destination_utf8: Option<String>,
    pub destination_bytes_hex: Option<String>,
    pub owner: Owner,
    pub reason: String,
    /// Closed reason code behind the entry, stable across releases.
    pub reason_code: PlanReason,
    /// Proof that allows this destination to enter the receipt.
    pub ownership_basis: OwnershipBasis,
    /// Legacy `source_id` when a verified lock entry proves the destination.
    pub source_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AssetChange {
    pub action: PlanAction,
    pub label: String,
}

#[derive(Debug)]
struct AggregatedChange {
    label: String,
    source_rank: u8,
    actions: BTreeSet<PlanAction>,
}

impl From<&PlanEntry> for OutputOperation {
    fn from(entry: &PlanEntry) -> Self {
        let (destination_utf8, destination_bytes_hex) = path_fields(&entry.destination);
        Self {
            action: entry.action,
            source: entry.source.clone(),
            destination_utf8,
            destination_bytes_hex,
            owner: entry.owner,
            reason: entry.message().to_owned(),
            reason_code: entry.reason,
            ownership_basis: entry.ownership_basis(),
            source_id: entry.ownership.source_id().cloned(),
        }
    }
}

pub(crate) fn asset_changes<'a>(
    entries: impl IntoIterator<Item = (PlanAction, &'a str)>,
) -> Vec<AssetChange> {
    let mut changes = BTreeMap::<String, AggregatedChange>::new();
    for (action, source) in entries {
        if action == PlanAction::Noop {
            continue;
        }
        let Some((key, label, source_rank)) = classify_asset(source) else {
            continue;
        };
        let change = changes.entry(key).or_insert_with(|| AggregatedChange {
            label: label.clone(),
            source_rank,
            actions: BTreeSet::new(),
        });
        if source_rank > change.source_rank {
            change.label = label;
            change.source_rank = source_rank;
            change.actions.clear();
        }
        if source_rank == change.source_rank {
            change.actions.insert(action);
        }
    }
    changes
        .into_values()
        .filter_map(|change| {
            representative_action(&change.actions).map(|action| AssetChange {
                action,
                label: change.label,
            })
        })
        .collect()
}

pub(crate) const fn pending_action_label(action: PlanAction) -> &'static str {
    match action {
        PlanAction::WriteReceipt => "Metadata",
        PlanAction::Create => "Restore",
        PlanAction::Update => "Update",
        PlanAction::Remove => "Remove",
        PlanAction::Adoptable => "Adopt",
        PlanAction::Drifted => "Drift",
        PlanAction::Conflict => "Conflict",
        PlanAction::RetainedUnmanaged => "Retain",
        PlanAction::RecoveryRequired => "Recover",
        PlanAction::Noop => "Keep",
    }
}

/// Text lines that name every provenance family a decision found.
///
/// Both counts stay separate and no line points at `adopt` unless a verified
/// candidate exists, so a matching unmanaged path always reads as unowned.
pub(crate) fn provenance_lines(summary: &PlanSummary, receipt_convergence: bool) -> Vec<String> {
    let mut lines = Vec::new();
    if summary.verified_legacy_candidates > 0 {
        lines.push(format!(
            "Adoptable  {} verified legacy {}; run adopt to transfer {}",
            summary.verified_legacy_candidates,
            plural(
                summary.verified_legacy_candidates,
                "candidate",
                "candidates"
            ),
            plural(summary.verified_legacy_candidates, "it", "them"),
        ));
    }
    if summary.matching_unmanaged > 0 {
        lines.push(format!(
            "Unowned    {} matching unmanaged {} without ownership proof; move or remove {}, then run plan again",
            summary.matching_unmanaged,
            plural(summary.matching_unmanaged, "path", "paths"),
            plural(summary.matching_unmanaged, "it", "them"),
        ));
    }
    if receipt_convergence {
        lines.push("Metadata   installation metadata will be reconciled".to_owned());
    }
    lines
}

const fn plural(count: usize, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 { one } else { many }
}

pub(crate) const fn completed_action_label(action: PlanAction) -> &'static str {
    match action {
        PlanAction::Create => "Restored",
        PlanAction::Update => "Updated",
        PlanAction::Remove => "Removed",
        PlanAction::RetainedUnmanaged => "Retained",
        _ => pending_action_label(action),
    }
}

fn classify_asset(source: &str) -> Option<(String, String, u8)> {
    let (source, activation) = source
        .strip_prefix("activation:claude:")
        .map_or((source, false), |source| (source, true));
    let source = source.strip_prefix("directory:").unwrap_or(source);
    // Containers and the receipt are not user-facing assets; the receipt is
    // reported as its own metadata line instead.
    if source.starts_with("container:") || source.starts_with("receipt:") {
        return None;
    }
    if let Some(path) = source.strip_prefix("skills/") {
        let name = path.split('/').next().filter(|name| !name.is_empty())?;
        return Some((
            format!("skill:{name}"),
            format!("Skill  {name}"),
            u8::from(!activation) + 1,
        ));
    }
    if activation {
        let name = source.split('/').next().filter(|name| !name.is_empty())?;
        return Some((format!("skill:{name}"), format!("Skill  {name}"), 1));
    }
    for (prefix, provider) in [
        ("agents/claude/", "Claude Code"),
        ("agents/codex/", "Codex"),
    ] {
        if let Some(path) = source.strip_prefix(prefix) {
            let name = file_stem(path);
            return Some((
                format!("agent:{provider}:{path}"),
                format!("Agent  {name} ({provider})"),
                2,
            ));
        }
    }
    if let Some(path) = source.strip_prefix("shared/claude/") {
        let name = file_stem(path);
        return Some((
            format!("support:claude:{path}"),
            format!("Support  {name} (Claude Code)"),
            2,
        ));
    }
    Some((format!("asset:{source}"), format!("Asset  {source}"), 2))
}

fn file_stem(path: &str) -> &str {
    path.rsplit('/')
        .next()
        .unwrap_or(path)
        .rsplit_once('.')
        .map_or(path.rsplit('/').next().unwrap_or(path), |(stem, _)| stem)
}

fn representative_action(actions: &BTreeSet<PlanAction>) -> Option<PlanAction> {
    for action in [
        PlanAction::Conflict,
        PlanAction::RecoveryRequired,
        PlanAction::Drifted,
        PlanAction::Adoptable,
        PlanAction::Update,
    ] {
        if actions.contains(&action) {
            return Some(action);
        }
    }
    if actions.contains(&PlanAction::Create) && actions.contains(&PlanAction::Remove) {
        return Some(PlanAction::Update);
    }
    [
        PlanAction::Remove,
        PlanAction::Create,
        PlanAction::RetainedUnmanaged,
    ]
    .into_iter()
    .find(|action| actions.contains(action))
}

#[derive(Debug, Serialize)]
pub struct Envelope {
    pub schema_version: u16,
    pub command: Option<String>,
    pub status: OutputStatus,
    pub exit_code: u8,
    pub catalog_version: String,
    pub transaction_id: Option<String>,
    pub providers: Vec<ProviderId>,
    pub summary: BTreeMap<String, usize>,
    pub operations: Vec<OutputOperation>,
    pub diagnostics: Vec<OutputDiagnostic>,
    pub data: Value,
    #[serde(skip)]
    pub suppress_human_output: bool,
}

impl Envelope {
    pub fn new(command: Option<&str>) -> Self {
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: command.map(str::to_owned),
            status: OutputStatus::Success,
            exit_code: SUCCESS_EXIT_CODE,
            catalog_version: env!("CARGO_PKG_VERSION").to_owned(),
            transaction_id: None,
            providers: Vec::new(),
            summary: BTreeMap::new(),
            operations: Vec::new(),
            diagnostics: Vec::new(),
            data: Value::Null,
            suppress_human_output: false,
        }
    }

    pub fn usage(command: Option<&str>, message: impl Into<String>) -> Self {
        let message = message.into();
        let mut envelope = Self::new(command);
        envelope.status = OutputStatus::Failed;
        envelope.exit_code = USAGE_EXIT_CODE;
        envelope.diagnostics.push(OutputDiagnostic::error(
            "usage",
            message,
            Some("Run the command with --help and provide the missing option.".to_owned()),
        ));
        envelope
    }

    pub fn failure(
        command: Option<&str>,
        status: OutputStatus,
        exit_code: u8,
        code: &str,
        message: impl Into<String>,
    ) -> Self {
        let mut envelope = Self::new(command);
        envelope.status = status;
        envelope.exit_code = exit_code;
        envelope
            .diagnostics
            .push(OutputDiagnostic::error(code, message, None));
        envelope
    }

    pub fn with_plan(mut self, plan: &Plan) -> Self {
        self.operations = plan.entries.iter().map(OutputOperation::from).collect();
        self.summary = summarize(plan);
        self.diagnostics
            .extend(plan.diagnostics.iter().map(|diagnostic| OutputDiagnostic {
                code: diagnostic.code.clone(),
                severity: match diagnostic.severity {
                    PlanSeverity::Error => OutputSeverity::Error,
                    PlanSeverity::Warning => OutputSeverity::Warning,
                },
                message: diagnostic.message.clone(),
                source_id: diagnostic.source_id.clone(),
                path_utf8: diagnostic.path_utf8.clone(),
                path_bytes_hex: diagnostic.path_bytes_hex.clone(),
                remediation: Some(plan_remediation(&diagnostic.code).to_owned()),
            }));
        if !plan.applicable {
            self.status = OutputStatus::Blocked;
            self.exit_code = CONFLICT_EXIT_CODE;
        } else if !plan.has_mutations() {
            self.status = OutputStatus::Noop;
        }
        if self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.path_bytes_hex.is_some())
        {
            self.status = OutputStatus::Failed;
            self.exit_code = ENVIRONMENT_EXIT_CODE;
        }
        self
    }
}

/// A matching path without proof is resolved by moving it aside, never by
/// adopting it: only a verified legacy entry can transfer ownership.
fn plan_remediation(code: &str) -> &'static str {
    if code == MATCHING_UNMANAGED_WITHOUT_PROOF {
        return "Move or remove this matching unmanaged path, then run plan again.";
    }
    if code == LEGACY_ENTRY_DOES_NOT_MATCH_CATALOG {
        return "Move or remove this legacy path, then run plan again; adopt only transfers entries that match the bundled catalog.";
    }
    "Resolve the reported path before applying this plan."
}

pub fn clap_envelope(command: Option<&str>, error: &clap::Error) -> Envelope {
    use clap::error::ErrorKind;

    let display = error.to_string();
    match error.kind() {
        ErrorKind::DisplayHelp => {
            let mut envelope = Envelope::new(command);
            envelope.data = json!({ "help": display });
            envelope
        }
        ErrorKind::DisplayVersion => {
            let mut envelope = Envelope::new(command);
            envelope.data = json!({ "version": display.trim_end() });
            envelope
        }
        _ => {
            let mut envelope = Envelope::usage(command, display.clone());
            envelope.data = json!({ "help": display });
            envelope
        }
    }
}

pub fn write_json(envelope: &Envelope, output: &mut impl Write) -> io::Result<()> {
    serde_json::to_writer(&mut *output, envelope).map_err(io::Error::other)?;
    writeln!(output)
}

pub fn write_human(envelope: &Envelope, output: &mut impl Write) -> io::Result<()> {
    write_human_with_detail(envelope, output, true)
}

pub fn write_human_compact(envelope: &Envelope, output: &mut impl Write) -> io::Result<()> {
    write_human_with_detail(envelope, output, false)
}

fn write_human_with_detail(
    envelope: &Envelope,
    output: &mut impl Write,
    detailed: bool,
) -> io::Result<()> {
    if envelope.data.get("kind").and_then(Value::as_str) == Some("upstream") {
        return write_upstream(envelope, output);
    }
    if envelope.data.get("result").and_then(Value::as_str) == Some("already_current")
        && let Some(message) = envelope.data.get("message").and_then(Value::as_str)
    {
        writeln!(output, "{message}")?;
        for diagnostic in &envelope.diagnostics {
            writeln!(output, "{}: {}", diagnostic.code, diagnostic.message)?;
        }
        return Ok(());
    }
    let committed = envelope.data.get("result").and_then(Value::as_str) == Some("committed");
    if !detailed && committed {
        writeln!(output, "Done")?;
        let summary = compact_summary(envelope);
        if !summary.is_empty() {
            writeln!(output, "  {}", summary.join("  · "))?;
        }
        for change in asset_changes(
            envelope
                .operations
                .iter()
                .map(|operation| (operation.action, operation.source.as_str())),
        ) {
            writeln!(
                output,
                "  {:<9} {}",
                completed_action_label(change.action),
                change.label
            )?;
        }
        for diagnostic in &envelope.diagnostics {
            let label = match diagnostic.severity {
                OutputSeverity::Info => "Info",
                OutputSeverity::Warning => "Note",
                OutputSeverity::Error => "Error",
            };
            writeln!(output, "  {label}  {}", diagnostic.message)?;
        }
        return Ok(());
    }
    if !envelope.operations.is_empty() {
        for (action, count) in &envelope.summary {
            writeln!(output, "{action}: {count}")?;
        }
        if envelope.command.is_some() {
            for operation in &envelope.operations {
                // The human line uses the same public action key as the JSON, so
                // the two surfaces can never drift apart.
                writeln!(
                    output,
                    "{} {} ({})",
                    action_key(operation.action),
                    operation
                        .destination_utf8
                        .as_deref()
                        .or(operation.destination_bytes_hex.as_deref())
                        .unwrap_or("<missing path>"),
                    operation.reason
                )?;
            }
        }
    }
    if !envelope.data.is_null() {
        match &envelope.data {
            Value::String(value) => writeln!(output, "{value}")?,
            value => writeln!(output, "{value}")?,
        }
    }
    for diagnostic in &envelope.diagnostics {
        writeln!(output, "{}: {}", diagnostic.code, diagnostic.message)?;
    }
    Ok(())
}

fn write_upstream(envelope: &Envelope, output: &mut impl Write) -> io::Result<()> {
    let action = envelope
        .data
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("check");
    let sources = envelope
        .data
        .get("sources")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let skills = envelope.data.get("skills").and_then(Value::as_array);
    let skill_count = skills.map_or(0, |items| items.len());
    let synced = envelope.data.get("result").and_then(Value::as_str) == Some("synced");
    let applied = envelope
        .data
        .get("applied")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    writeln!(output, "Upstream {action}")?;
    writeln!(output, "Sources {sources}  · Skills {skill_count}")?;

    if let Some(skills) = skills {
        for skill in skills {
            let reported_state = skill
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let name = skill
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            let state = if synced && applied.contains(name) {
                "updated"
            } else {
                reported_state
            };
            if state == "current" {
                continue;
            }
            let source = skill
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            writeln!(output, "{state}  {name}  ({source})")?;
        }
    }
    if synced {
        writeln!(output, "Applied {} upstream updates.", applied.len())?;
    } else if envelope.status == OutputStatus::Noop {
        writeln!(
            output,
            "Every vendored skill matches its pinned upstream tree."
        )?;
    }
    for diagnostic in &envelope.diagnostics {
        writeln!(output, "{}: {}", diagnostic.code, diagnostic.message)?;
    }
    Ok(())
}

pub(crate) fn compact_summary(envelope: &Envelope) -> Vec<String> {
    const ACTIONS: [(&str, &str); 11] = [
        ("create", "created"),
        ("update", "updated"),
        ("remove", "removed"),
        ("adoptable", "adoptable"),
        ("matching_unmanaged", "matching unmanaged"),
        ("drifted", "drifted"),
        ("conflict", "conflicting"),
        ("retained_unmanaged", "retained"),
        ("recovery_required", "requiring recovery"),
        ("write_receipt", "metadata reconciled"),
        ("noop", "unchanged"),
    ];
    ACTIONS
        .iter()
        .filter_map(|(action, label)| {
            envelope
                .summary
                .get(*action)
                .filter(|count| **count > 0)
                .map(|count| format!("{count} {label}"))
        })
        .collect()
}

/// Projects the decision summary and keeps the two collision families apart, so
/// a verified candidate is never counted as a foreign path.
fn summarize(plan: &Plan) -> BTreeMap<String, usize> {
    let PlanSummary {
        mut actions,
        verified_legacy_candidates,
        matching_unmanaged,
    } = plan.summary();
    if verified_legacy_candidates > 0 {
        actions.insert(
            "verified_legacy_candidates".to_owned(),
            verified_legacy_candidates,
        );
    }
    if matching_unmanaged > 0 {
        actions.insert("matching_unmanaged".to_owned(), matching_unmanaged);
    }
    actions
}

pub fn path_fields(path: &Path) -> (Option<String>, Option<String>) {
    match path.to_str() {
        Some(path) => (Some(path.to_owned()), None),
        None => (None, Some(hex(&path_key(path.as_os_str())))),
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(all(test, unix))]
mod tests {
    use std::ffi::OsString;
    use std::io::{self, Write};
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    use serde_json::Value;

    use super::{
        ENVIRONMENT_EXIT_CODE, Envelope, MATCHING_UNMANAGED_WITHOUT_PROOF, OutputDiagnostic,
        OutputOperation, OutputSeverity, OutputStatus, USAGE_EXIT_CODE, asset_changes, path_fields,
        write_human, write_human_compact, write_json,
    };
    use crate::plan::{
        Diagnostic, DiagnosticSeverity, Owner, OwnershipBasis, OwnershipClaim, Plan, PlanAction,
        PlanEntry, PlanReason, PlannedMutation,
    };

    #[test]
    fn json_envelope_always_contains_the_v1_contract() {
        let envelope = Envelope::usage(None, "provide --provider");
        let mut bytes = Vec::new();
        assert!(write_json(&envelope, &mut bytes).is_ok());
        let parsed = serde_json::from_slice::<Value>(&bytes);
        let Ok(Value::Object(object)) = parsed else {
            panic!("output was not one JSON object");
        };
        assert_eq!(object.len(), 11);
        assert_eq!(object["schema_version"], 1);
        assert_eq!(object["exit_code"], USAGE_EXIT_CODE);
        for field in [
            "command",
            "status",
            "catalog_version",
            "transaction_id",
            "providers",
            "summary",
            "operations",
            "diagnostics",
            "data",
        ] {
            assert!(object.contains_key(field));
        }
    }

    #[test]
    fn plan_diagnostics_and_human_output_preserve_paths_and_status() {
        let non_utf8 = PathBuf::from(OsString::from_vec(b"/tmp/path-\xff".to_vec()));
        let plan = Plan {
            schema_version: 1,
            applicable: false,
            entries: vec![PlanEntry {
                action: PlanAction::Conflict,
                source: "skill:test".to_owned(),
                destination: non_utf8.clone(),
                owner: Owner::Unmanaged,
                reason: PlanReason::UnmanagedConflict,
                ownership: OwnershipClaim::None,
            }],
            operations: Vec::<PlannedMutation>::new(),
            diagnostics: vec![
                Diagnostic {
                    code: "unsafe_path".to_owned(),
                    severity: DiagnosticSeverity::Error,
                    message: "unsafe destination".to_owned(),
                    source_id: None,
                    path_utf8: None,
                    path_bytes_hex: Some("ff".to_owned()),
                },
                Diagnostic {
                    code: "notice".to_owned(),
                    severity: DiagnosticSeverity::Warning,
                    message: "review destination".to_owned(),
                    source_id: None,
                    path_utf8: Some("/tmp/path".to_owned()),
                    path_bytes_hex: None,
                },
            ],
        };
        let mut envelope = Envelope::new(Some("plan")).with_plan(&plan);
        assert_eq!(envelope.status, OutputStatus::Failed);
        assert_eq!(envelope.exit_code, ENVIRONMENT_EXIT_CODE);
        assert_eq!(envelope.summary["conflict"], 1);
        assert!(envelope.operations[0].destination_utf8.is_none());
        assert!(envelope.operations[0].destination_bytes_hex.is_some());

        envelope.operations.push(OutputOperation {
            action: PlanAction::Noop,
            source: "missing".to_owned(),
            destination_utf8: None,
            destination_bytes_hex: None,
            owner: Owner::ArthurWorkflow,
            reason: PlanReason::DestinationAbsent.message().to_owned(),
            reason_code: PlanReason::DestinationAbsent,
            ownership_basis: OwnershipBasis::None,
            source_id: None,
        });
        envelope.data = Value::String("done".to_owned());
        let mut output = Vec::new();
        assert!(write_human(&envelope, &mut output).is_ok());
        let output = String::from_utf8_lossy(&output);
        assert!(output.contains("<missing path>"));
        assert!(output.contains("done"));
        assert!(output.contains("unsafe_path: unsafe destination"));
        assert_eq!(path_fields(&non_utf8).0, None);
    }

    #[test]
    fn a_matching_unmanaged_path_is_never_remediated_by_adopt() {
        let plan = Plan {
            schema_version: 1,
            applicable: false,
            entries: vec![PlanEntry {
                action: PlanAction::Conflict,
                source: "skills/example/SKILL.md".to_owned(),
                destination: PathBuf::from("/home/user/.agents/skills/example/SKILL.md"),
                owner: Owner::Unmanaged,
                reason: PlanReason::MatchingUnmanagedWithoutProof,
                ownership: OwnershipClaim::None,
            }],
            operations: Vec::<PlannedMutation>::new(),
            diagnostics: vec![Diagnostic {
                code: MATCHING_UNMANAGED_WITHOUT_PROOF.to_owned(),
                severity: DiagnosticSeverity::Error,
                message: "matching unmanaged asset has no ownership proof".to_owned(),
                source_id: None,
                path_utf8: Some("/home/user/.agents/skills/example/SKILL.md".to_owned()),
                path_bytes_hex: None,
            }],
        };

        let envelope = Envelope::new(Some("plan")).with_plan(&plan);

        assert_eq!(envelope.status, OutputStatus::Blocked);
        let diagnostic = &envelope.diagnostics[0];
        assert_eq!(
            diagnostic.remediation.as_deref(),
            Some("Move or remove this matching unmanaged path, then run plan again.")
        );
        let mut output = Vec::new();
        assert!(write_human(&envelope, &mut output).is_ok());
        let rendered = String::from_utf8_lossy(&output);
        assert!(rendered.contains("matching unmanaged"));
        assert!(!rendered.contains("adopt"), "{rendered}");
    }

    #[test]
    fn compact_committed_output_summarizes_without_listing_paths() {
        let mut envelope = Envelope::new(Some("install"));
        envelope.summary.insert("noop".to_owned(), 513);
        envelope.summary.insert("update".to_owned(), 13);
        envelope.operations.push(OutputOperation {
            action: PlanAction::Update,
            source: "skills/test/SKILL.md".to_owned(),
            destination_utf8: Some("/home/user/.agents/skills/test".to_owned()),
            destination_bytes_hex: None,
            owner: Owner::ArthurWorkflow,
            reason: PlanReason::EligibleUpdate.message().to_owned(),
            reason_code: PlanReason::EligibleUpdate,
            ownership_basis: OwnershipBasis::Receipt,
            source_id: Some("skills/example/SKILL.md".to_owned()),
        });
        envelope.data = serde_json::json!({ "applied": true, "result": "committed" });
        envelope.diagnostics.push(OutputDiagnostic {
            code: "codex_uses_implicit_skills".to_owned(),
            severity: OutputSeverity::Warning,
            message: "Codex reads shared skills directly.".to_owned(),
            source_id: None,
            path_utf8: None,
            path_bytes_hex: None,
            remediation: None,
        });

        let mut output = Vec::new();
        assert!(write_human_compact(&envelope, &mut output).is_ok());
        assert_eq!(
            String::from_utf8_lossy(&output),
            "Done\n  13 updated  · 513 unchanged\n  Updated   Skill  test\n  Note  Codex reads shared skills directly.\n"
        );
    }

    #[test]
    fn asset_changes_group_files_and_provider_activations_by_managed_asset() {
        let changes = asset_changes([
            (PlanAction::Update, "skills/coss/SKILL.md"),
            (PlanAction::Create, "directory:skills/coss/references"),
            (PlanAction::Create, "activation:claude:coss"),
            (PlanAction::Create, "agents/codex/docs-researcher.toml"),
            (PlanAction::Create, "container:codex-agents"),
            (PlanAction::Noop, "skills/current/SKILL.md"),
        ]);

        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].action, PlanAction::Create);
        assert_eq!(changes[0].label, "Agent  docs-researcher (Codex)");
        assert_eq!(changes[1].action, PlanAction::Update);
        assert_eq!(changes[1].label, "Skill  coss");
    }

    #[test]
    fn upstream_human_output_lists_changes_and_results() {
        let mut envelope = Envelope::new(Some("upstream"));
        envelope.data = serde_json::json!({
            "kind": "upstream",
            "action": "check",
            "sources": 1,
            "skills": [
                { "name": "alpha", "source": "owner/repository", "state": "current" },
                {
                    "name": "beta",
                    "source": "owner/repository",
                    "state": "update_available"
                }
            ],
            "result": null,
            "applied": []
        });
        let mut output = Vec::new();
        assert!(write_human(&envelope, &mut output).is_ok());
        let output = String::from_utf8_lossy(&output);
        assert!(output.contains("Upstream check"));
        assert!(output.contains("Sources 1  · Skills 2"));
        assert!(output.contains("update_available  beta  (owner/repository)"));
        assert!(!output.contains("alpha"));

        envelope.data["result"] = Value::String("synced".to_owned());
        envelope.data["applied"] = serde_json::json!(["beta"]);
        let mut output = Vec::new();
        assert!(write_human(&envelope, &mut output).is_ok());
        let output = String::from_utf8_lossy(&output);
        assert!(output.contains("updated  beta  (owner/repository)"));
        assert!(output.contains("Applied 1 upstream updates."));

        envelope.status = OutputStatus::Noop;
        envelope.data["result"] = Value::Null;
        let mut output = Vec::new();
        assert!(write_human(&envelope, &mut output).is_ok());
        assert!(
            String::from_utf8_lossy(&output)
                .contains("Every vendored skill matches its pinned upstream tree.")
        );
    }

    #[test]
    fn writers_propagate_output_failures() {
        struct Reject;

        impl Write for Reject {
            fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("closed"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("closed"))
            }
        }

        let envelope = Envelope::usage(Some("install"), "missing provider");
        assert!(write_json(&envelope, &mut Reject).is_err());
        assert!(write_human(&envelope, &mut Reject).is_err());
        assert!(write_human_compact(&envelope, &mut Reject).is_err());
    }
}
