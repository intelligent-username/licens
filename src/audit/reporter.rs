use crate::audit::model::{AuditReport, OutputFormat};

pub struct AuditReporter;

impl AuditReporter {
    pub fn report(report: &AuditReport, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
            OutputFormat::Markdown => Self::render_markdown(report),
        }
    }

    fn render_markdown(report: &AuditReport) -> String {
        let mut out = String::new();

        out.push_str("# Dependency License Audit Report\n\n");

        if let Some(ref proj_lic) = report.project_license {
            out.push_str(&format!("**Project License**: `{proj_lic}`\n\n"));
        } else {
            out.push_str("**Project License**: *None detected*\n\n");
        }

        out.push_str(&format!(
            "Audited **{}** dependencies across manifests.\n\n",
            report.audited_dependencies
        ));

        // Conflicts
        if !report.conflicts.is_empty() {
            out.push_str("## ⚠️ Conflicts & Warnings\n\n");
            for c in &report.conflicts {
                out.push_str(&format!(
                    "- **{}** (`{}`): {}\n",
                    c.dependency_name, c.dependency_license, c.reason
                ));
            }
            out.push('\n');
        } else {
            out.push_str("## ✅ No License Conflicts Detected\n\n");
        }

        // Table
        out.push_str("## Dependency License Inventory\n\n");
        out.push_str("| Package | Version | Type | Manifest | License |\n");
        out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

        for dep in &report.dependencies {
            let ver = dep.version.as_deref().unwrap_or("-");
            let kind = match dep.kind {
                crate::audit::model::DependencyKind::Normal => "prod",
                crate::audit::model::DependencyKind::Dev => "dev",
                crate::audit::model::DependencyKind::Build => "build",
            };
            let lic = dep.license.as_deref().unwrap_or("Unknown");
            out.push_str(&format!(
                "| {} | {} | {} | {} | `{}` |\n",
                dep.name, ver, kind, dep.source_manifest, lic
            ));
        }

        out.push_str("\n## Recommendations\n\n");
        out.push_str(&format!(
            "- **Most Permissive License**: `{}`\n",
            report.recommended_permissive
        ));
        out.push_str(&format!(
            "- **Most Restrictive / Copyleft**: `{}`\n",
            report.recommended_restrictive
        ));

        out
    }
}
