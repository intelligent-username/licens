use crate::audit::{
    AuditReport, AuditReporter, CompatibilityAnalyzer, LicenseResolver, ManifestScanner,
    OutputFormat,
};
use crate::error::Result;

pub struct AuditOptions {
    pub format: OutputFormat,
    pub include_dev: bool,
    pub include_build: bool,
}

pub async fn run(opts: AuditOptions) -> Result<()> {
    let scanned = ManifestScanner::scan_current_dir(opts.include_dev, opts.include_build)?;

    let resolver = LicenseResolver::new();
    let total_deps = scanned.dependencies.len();
    let resolved_deps = resolver.resolve_all(scanned.dependencies).await;

    let (conflicts, rec_permissive, rec_restrictive) =
        CompatibilityAnalyzer::analyze(scanned.detected_license.as_deref(), &resolved_deps);

    let report = AuditReport {
        project_license: scanned.detected_license,
        total_dependencies: total_deps,
        audited_dependencies: resolved_deps.len(),
        dependencies: resolved_deps,
        conflicts,
        recommended_permissive: rec_permissive,
        recommended_restrictive: rec_restrictive,
    };

    let output = AuditReporter::report(&report, opts.format);
    println!("{output}");

    Ok(())
}
