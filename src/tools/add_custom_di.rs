// Add Custom DI Tool Implementation
// This module provides functionality to add custom DI definitions via YAML format

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use spec_engine::{create_dynamic_catalog, DynamicCatalog};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const DEFAULT_REGION: &str = "南网";

// ============================================================================
// Data Structures
// ============================================================================

/// Input structure for add_custom_di tool
#[derive(Debug, Deserialize)]
pub struct AddCustomDiInput {
    /// YAML content as string (must be an array of DI definitions)
    pub yaml_content: String,
    
    /// Force overwrite existing definitions (default: false)
    #[serde(default)]
    pub force: bool,
    
    /// Dry run mode - validate only, don't persist (default: false)
    #[serde(default)]
    pub dry_run: bool,
}

/// Output structure for add_custom_di tool
#[derive(Debug, Serialize)]
pub struct AddCustomDiOutput {
    /// Operation success status
    pub success: bool,
    
    /// Action taken: "added", "would_add" (dry run), "error"
    pub action: String,
    
    /// Human-readable message
    pub message: String,
    
    /// Number of DI definitions processed per protocol
    #[serde(skip_serializing_if = "Option::is_none")]
    pub di_count: Option<HashMap<String, usize>>,
    
    /// Files written (protocol -> file path)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_written: Option<HashMap<String, String>>,
    
    /// Conflicts detected (only when force=false and conflicts exist)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts: Option<Vec<DiConflict>>,
    
    /// Detailed errors if validation or processing failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

/// Conflict information for a DI definition
#[derive(Debug, Serialize)]
pub struct DiConflict {
    pub di_id: String,
    pub protocol: String,
    pub region: Option<String>,
    pub existing_name: String,
    pub new_name: String,
}

/// Parsed DI definition from YAML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiDefinition {
    /// DI code (e.g., "00010000")
    pub id: String,
    
    /// DI name
    pub name: String,
    
    /// Protocol (e.g., "dlt645-2007", "csg13")
    pub protocol: String,
    
    /// Optional region codes (e.g., ["南网", "广东"])
    #[serde(default)]
    pub region: Option<Vec<String>>,
    
    /// Preserve all other fields as-is
    #[serde(flatten)]
    pub extra: HashMap<String, YamlValue>,
}

// ============================================================================
// Main Entry Point
// ============================================================================

/// Main entry point for adding custom DI definitions
pub fn add_custom_di(input: AddCustomDiInput) -> Result<AddCustomDiOutput> {
    // 1. Parse YAML array
    let dis = parse_di_array(&input.yaml_content)?;

    // 2. Basic validation (required fields, ID format)
    validate_basic(&dis)?;

    // 3. Group by protocol
    let grouped = group_by_protocol(dis);

    // 4. Validate with spec-compiler
    validate_with_compiler(&grouped)?;

    // 5. Check conflicts
    let catalog = create_dynamic_catalog();
    let conflicts = check_conflicts(&grouped, &catalog)?;

    if !conflicts.is_empty() && !input.force {
        return Ok(create_conflict_output(conflicts));
    }

    // 6. Handle dry run
    if input.dry_run {
        return Ok(create_dry_run_output(&grouped, conflicts));
    }

    // 7. Write to files
    let files = write_to_files(&grouped)?;

    // 8. Return success
    Ok(create_success_output(&grouped, files, conflicts))
}

// ============================================================================
// YAML Parsing
// ============================================================================

/// Parse YAML content into DiDefinition array
fn parse_di_array(yaml_content: &str) -> Result<Vec<DiDefinition>> {
    if yaml_content.trim().is_empty() {
        bail!("YAML content is empty");
    }

    let value: YamlValue = serde_yaml::from_str(yaml_content)
        .context("Failed to parse YAML")?;

    let array = value
        .as_sequence()
        .ok_or_else(|| {
            anyhow!(
                "Expected YAML array at root, got {}",
                value_type_name(&value)
            )
        })?;

    let mut definitions = Vec::new();
    for (idx, item) in array.iter().enumerate() {
        let def: DiDefinition = serde_yaml::from_value(item.clone())
            .with_context(|| format!("Failed to parse DI definition at index {}", idx))?;
        definitions.push(def);
    }

    Ok(definitions)
}

fn value_type_name(value: &YamlValue) -> &'static str {
    match value {
        YamlValue::Null => "null",
        YamlValue::Bool(_) => "boolean",
        YamlValue::Number(_) => "number",
        YamlValue::String(_) => "string",
        YamlValue::Sequence(_) => "array",
        YamlValue::Mapping(_) => "object",
        YamlValue::Tagged(_) => "tagged",
    }
}

// ============================================================================
// Basic Validation
// ============================================================================

/// Validate required fields and basic format
fn validate_basic(dis: &[DiDefinition]) -> Result<()> {
    let mut errors = Vec::new();

    for (idx, di) in dis.iter().enumerate() {
        // Check required fields
        if di.id.is_empty() {
            errors.push(format!("DI at index {}: missing 'id' field", idx));
        }
        if di.name.is_empty() {
            errors.push(format!("DI at index {}: missing 'name' field", idx));
        }
        if di.protocol.is_empty() {
            errors.push(format!("DI at index {}: missing 'protocol' field", idx));
        }

        // Validate ID format (hex string, 8 chars)
        if !di.id.is_empty() && !is_valid_di_id(&di.id) {
            errors.push(format!(
                "DI at index {}: invalid ID format '{}' (expected 8-char hex string)",
                idx, di.id
            ));
        }
    }

    if !errors.is_empty() {
        bail!("Validation errors:\n{}", errors.join("\n"));
    }

    Ok(())
}

fn is_valid_di_id(id: &str) -> bool {
    id.len() == 8 && id.chars().all(|c| c.is_ascii_hexdigit())
}

// ============================================================================
// Group by Protocol
// ============================================================================

/// Group DI definitions by protocol
fn group_by_protocol(dis: Vec<DiDefinition>) -> HashMap<String, Vec<DiDefinition>> {
    let mut grouped: HashMap<String, Vec<DiDefinition>> = HashMap::new();

    for di in dis {
        grouped
            .entry(di.protocol.clone())
            .or_insert_with(Vec::new)
            .push(di);
    }

    grouped
}

// ============================================================================
// Compiler Validation
// ============================================================================

/// Validate YAML structure using spec-compiler
fn validate_with_compiler(grouped: &HashMap<String, Vec<DiDefinition>>) -> Result<()> {
    use spec_compiler::Compiler;
    use tempfile::TempDir;

    let mut errors = Vec::new();

    for (protocol, protocol_dis) in grouped {
        // Create temporary directory
        let temp_dir = TempDir::new()
            .context("Failed to create temporary directory")?;

        // Create protocol subdirectory (spec-compiler requires this)
        let protocol_dir = temp_dir.path().join(protocol);
        std::fs::create_dir_all(&protocol_dir)
            .context("Failed to create protocol subdirectory")?;

        // Create protocol YAML file in the subdirectory
        let protocol_yaml = create_protocol_yaml(protocol, protocol_dis)?;
        let temp_file = protocol_dir.join("test.yaml");
        std::fs::write(&temp_file, protocol_yaml)
            .context("Failed to write temporary YAML file")?;

        // Compile to validate
        let compiler = Compiler::new();
        match compiler.compile_schema_dir(temp_dir.path()) {
            Ok(_) => {
                // Validation passed
            }
            Err(e) => {
                errors.push(format!("Protocol '{}' validation failed: {}", protocol, e));
            }
        }
    }

    if !errors.is_empty() {
        bail!("Compiler validation errors:\n{}", errors.join("\n"));
    }

    Ok(())
}

fn create_protocol_yaml(protocol: &str, dis: &[DiDefinition]) -> Result<String> {
    let mut mapping = serde_yaml::Mapping::new();
    mapping.insert(
        YamlValue::String("protocol".to_string()),
        YamlValue::String(protocol.to_string()),
    );
    mapping.insert(
        YamlValue::String("data_items".to_string()),
        serde_yaml::to_value(dis)?,
    );

    let yaml_string = serde_yaml::to_string(&YamlValue::Mapping(mapping))?;
    Ok(yaml_string)
}

// ============================================================================
// Conflict Detection
// ============================================================================

/// Check for conflicts with existing DI definitions
fn check_conflicts(
    grouped: &HashMap<String, Vec<DiDefinition>>,
    catalog: &DynamicCatalog,
) -> Result<Vec<DiConflict>> {
    let mut conflicts = Vec::new();
    let default_regions = vec![DEFAULT_REGION.to_string()];

    for (protocol, dis) in grouped {
        for di in dis {
            let di_code = u32::from_str_radix(&di.id, 16)
                .with_context(|| format!("Invalid DI code: {}", di.id))?;

            // Check each region (or default region if none specified)
            let regions = di
                .region
                .as_ref()
                .map(|r| r.as_slice())
                .unwrap_or(&default_regions);

            for region in regions {
                // Look up in catalog
                if let Some(existing) = catalog.lookup(protocol, di_code, region, None) {
                    conflicts.push(DiConflict {
                        di_id: di.id.clone(),
                        protocol: protocol.clone(),
                        region: Some(region.clone()),
                        existing_name: existing.name.clone(),
                        new_name: di.name.clone(),
                    });
                }
            }
        }
    }

    Ok(conflicts)
}

// ============================================================================
// File Management
// ============================================================================

/// Get user_def directory path
fn get_user_def_path() -> PathBuf {
    // 1. Check environment variable
    if let Ok(path) = std::env::var("SPEC_ENGINE_USER_DEF_PATH") {
        return PathBuf::from(path);
    }

    // 2. Use workspace path
    let workspace_path = PathBuf::from("./user_def");
    if workspace_path.exists() || std::env::current_dir().is_ok() {
        return workspace_path;
    }

    // 3. Use global config directory
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("spec-engine-mcp")
        .join("user_def")
}

/// Write DI definitions to user_def files
fn write_to_files(
    grouped: &HashMap<String, Vec<DiDefinition>>,
) -> Result<HashMap<String, String>> {
    let user_def_dir = get_user_def_path();

    // Ensure directory exists
    if !user_def_dir.exists() {
        std::fs::create_dir_all(&user_def_dir)
            .context("Failed to create user_def directory")?;
    }

    let mut files_written = HashMap::new();

    for (protocol, new_dis) in grouped {
        let file_path = user_def_dir.join(format!("{}.yaml", protocol));

        // Read existing definitions if file exists
        let mut existing_dis = if file_path.exists() {
            read_existing_definitions(&file_path)?
        } else {
            Vec::new()
        };

        // Merge: replace matching IDs, append new ones
        merge_definitions(&mut existing_dis, new_dis);

        // Write to file
        write_yaml_file(&file_path, protocol, &existing_dis)?;

        files_written.insert(
            protocol.clone(),
            file_path.to_string_lossy().to_string(),
        );
    }

    Ok(files_written)
}

/// Read existing DI definitions from file
fn read_existing_definitions(path: &Path) -> Result<Vec<DiDefinition>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let yaml: YamlValue = serde_yaml::from_str(&content)
        .context("Failed to parse existing YAML")?;

    // Extract data_items array
    let data_items = yaml
        .get("data_items")
        .and_then(|v| v.as_sequence())
        .ok_or_else(|| anyhow!("Expected data_items array in {}", path.display()))?;

    let mut definitions = Vec::new();
    for item in data_items {
        let def: DiDefinition = serde_yaml::from_value(item.clone())
            .context("Failed to parse existing DI definition")?;
        definitions.push(def);
    }

    Ok(definitions)
}

/// Merge new definitions into existing ones
fn merge_definitions(existing: &mut Vec<DiDefinition>, new_dis: &[DiDefinition]) {
    for new_di in new_dis {
        // Find matching definition by ID
        if let Some(pos) = existing.iter().position(|e| e.id == new_di.id) {
            // Replace existing
            existing[pos] = new_di.clone();
        } else {
            // Append new
            existing.push(new_di.clone());
        }
    }
}

/// Write YAML file with proper structure
fn write_yaml_file(path: &Path, protocol: &str, definitions: &[DiDefinition]) -> Result<()> {
    // Create file structure
    let mut mapping = serde_yaml::Mapping::new();
    mapping.insert(
        YamlValue::String("protocol".to_string()),
        YamlValue::String(protocol.to_string()),
    );
    mapping.insert(
        YamlValue::String("data_items".to_string()),
        serde_yaml::to_value(definitions)?,
    );

    let yaml_string = serde_yaml::to_string(&YamlValue::Mapping(mapping))
        .context("Failed to serialize YAML")?;

    // Write atomically (write to temp, then rename)
    let temp_path = path.with_extension("yaml.tmp");
    std::fs::write(&temp_path, yaml_string)
        .with_context(|| format!("Failed to write {}", temp_path.display()))?;

    std::fs::rename(&temp_path, path).with_context(|| {
        format!(
            "Failed to rename {} to {}",
            temp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

// ============================================================================
// Output Helpers
// ============================================================================

fn create_success_output(
    grouped: &HashMap<String, Vec<DiDefinition>>,
    files: HashMap<String, String>,
    conflicts: Vec<DiConflict>,
) -> AddCustomDiOutput {
    let total_count: usize = grouped.values().map(|v| v.len()).sum();
    let di_count: HashMap<String, usize> = grouped.iter().map(|(k, v)| (k.clone(), v.len())).collect();

    let action = if conflicts.is_empty() {
        "added"
    } else {
        "added_with_overwrites"
    };

    let message = if conflicts.is_empty() {
        format!(
            "Successfully added {} DI definition(s) across {} protocol(s)",
            total_count,
            grouped.len()
        )
    } else {
        format!(
            "Successfully added {} DI definition(s) across {} protocol(s), overwriting {} existing definition(s)",
            total_count,
            grouped.len(),
            conflicts.len()
        )
    };

    AddCustomDiOutput {
        success: true,
        action: action.to_string(),
        message,
        di_count: Some(di_count),
        files_written: Some(files),
        conflicts: if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        },
        errors: None,
    }
}

fn create_dry_run_output(
    grouped: &HashMap<String, Vec<DiDefinition>>,
    conflicts: Vec<DiConflict>,
) -> AddCustomDiOutput {
    let total_count: usize = grouped.values().map(|v| v.len()).sum();
    let di_count: HashMap<String, usize> = grouped.iter().map(|(k, v)| (k.clone(), v.len())).collect();

    let files_would_write: HashMap<String, String> = grouped
        .keys()
        .map(|protocol| (protocol.clone(), format!("user_def/{}.yaml", protocol)))
        .collect();

    let message = if conflicts.is_empty() {
        format!(
            "Dry run: would add {} DI definition(s) across {} protocol(s)",
            total_count,
            grouped.len()
        )
    } else {
        format!(
            "Dry run: would add {} DI definition(s) across {} protocol(s), overwriting {} existing definition(s)",
            total_count,
            grouped.len(),
            conflicts.len()
        )
    };

    AddCustomDiOutput {
        success: true,
        action: "would_add".to_string(),
        message,
        di_count: Some(di_count),
        files_written: Some(files_would_write),
        conflicts: if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        },
        errors: None,
    }
}

fn create_conflict_output(conflicts: Vec<DiConflict>) -> AddCustomDiOutput {
    let error_messages: Vec<String> = conflicts
        .iter()
        .map(|c| {
            format!(
                "DI {} in protocol '{}'{}: existing='{}', new='{}'",
                c.di_id,
                c.protocol,
                c.region
                    .as_ref()
                    .map(|r| format!(" region '{}'", r))
                    .unwrap_or_default(),
                c.existing_name,
                c.new_name
            )
        })
        .collect();

    AddCustomDiOutput {
        success: false,
        action: "error".to_string(),
        message: format!(
            "Conflicts detected: {} DI(s) already exist. Use force=true to overwrite.",
            conflicts.len()
        ),
        di_count: None,
        files_written: None,
        conflicts: Some(conflicts),
        errors: Some(error_messages),
    }
}
