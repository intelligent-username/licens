pub mod compatibility;
pub mod model;
pub mod reporter;
pub mod resolver;
pub mod scanner;

pub use compatibility::CompatibilityAnalyzer;
pub use model::{AuditReport, Dependency, DependencyKind, OutputFormat};
pub use reporter::AuditReporter;
pub use resolver::LicenseResolver;
pub use scanner::ManifestScanner;
