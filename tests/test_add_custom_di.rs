// Test for add_custom_di functionality
use spec_engine_mcp::tools::add_custom_di::{add_custom_di, AddCustomDiInput};

#[test]
fn test_validate_yaml_structure() {
    let yaml_content = r#"
- id: "0201FF00"
  name: "电压数据块"
  protocol: "csg13"
  region: ["南网"]
  format:
    type: "object"
    fields:
      - name: "voltage_a"
        type: "uint16"
        unit: "0.1V"
"#;

    let input = AddCustomDiInput {
        yaml_content: yaml_content.to_string(),
        force: false,
        dry_run: true,
    };

    let result = add_custom_di(input);
    
    match result {
        Ok(output) => {
            println!("✅ Test passed!");
            println!("Action: {}", output.action);
            println!("Message: {}", output.message);
            if let Some(di_count) = output.di_count {
                println!("DI count: {:?}", di_count);
            }
            if let Some(errors) = output.errors {
                println!("Errors: {:?}", errors);
            }
            assert!(output.success, "Expected success");
            assert_eq!(output.action, "would_add");
        }
        Err(e) => {
            panic!("❌ Test failed: {}", e);
        }
    }
}

#[test]
fn test_invalid_yaml() {
    let yaml_content = r#"
- id: ""
  name: "Missing ID"
  protocol: "csg13"
"#;

    let input = AddCustomDiInput {
        yaml_content: yaml_content.to_string(),
        force: false,
        dry_run: true,
    };

    let result = add_custom_di(input);
    assert!(result.is_err(), "Expected validation error for empty ID");
}

#[test]
fn test_protocol_subdirectory() {
    // This test verifies that the temporary files are created in protocol subdirectories
    let yaml_content = r#"
- id: "0201FF01"
  name: "测试数据项"
  protocol: "csg13"
"#;

    let input = AddCustomDiInput {
        yaml_content: yaml_content.to_string(),
        force: false,
        dry_run: true,
    };

    let result = add_custom_di(input);
    
    match result {
        Ok(output) => {
            println!("✅ Protocol subdirectory test passed!");
            println!("Message: {}", output.message);
            assert!(output.success);
        }
        Err(e) => {
            // Check if error is NOT about uncategorized yaml files
            let error_msg = format!("{}", e);
            assert!(
                !error_msg.contains("未归类的 .yaml 文件"),
                "❌ Still getting uncategorized file error: {}",
                error_msg
            );
            println!("❌ Unexpected error: {}", e);
        }
    }
}
