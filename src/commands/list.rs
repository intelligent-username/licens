use crate::error::Result;
use crate::templates::LICENSES;

pub fn run() -> Result<()> {
    println!("{:<15} {:<15} {:<18} {}", "KEY", "SPDX ID", "CATEGORY", "NAME");
    println!("{}", "-".repeat(70));

    for lic in LICENSES {
        println!(
            "{:<15} {:<15} {:<18} {}",
            lic.key,
            lic.spdx_id,
            lic.category.as_str(),
            lic.name
        );
    }

    println!("\nUse 'licens <KEY>' or 'licens generate <KEY>' to create a LICENSE file.");
    Ok(())
}
