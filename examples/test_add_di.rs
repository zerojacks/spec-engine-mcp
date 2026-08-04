// Standalone test for add_custom_di

fn main() {
    use spec_engine_mcp::add_custom_di;
    
    println!("🧪 Testing add_custom_di with protocol subdirectory fix...\n");

    let test_yaml = r#"
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
- id: "0201FF01"
  name: "电流数据块"
  protocol: "csg13"
  region: ["南网"]
"#;

    let input = serde_json::json!({
        "yaml_content": test_yaml,
        "force": false,
        "dry_run": true
    });
    
    let input_struct: spec_engine_mcp::AddCustomDiInput = 
        serde_json::from_value(input).expect("Failed to parse input");

    println!("📝 Input YAML:");
    println!("{}", test_yaml);
    println!("\n🔄 Processing...\n");

    match add_custom_di(input_struct) {
        Ok(output) => {
            println!("✅ SUCCESS!");
            println!("   Action: {}", output.action);
            println!("   Message: {}", output.message);
            
            if let Some(di_count) = &output.di_count {
                println!("\n📊 DI Count:");
                for (protocol, count) in di_count {
                    println!("   {} -> {} definitions", protocol, count);
                }
            }
            
            if let Some(files) = &output.files_written {
                println!("\n📁 Files to be written:");
                for (protocol, path) in files {
                    println!("   {} -> {}", protocol, path);
                }
            }
            
            if let Some(conflicts) = &output.conflicts {
                println!("\n⚠️  Conflicts detected:");
                for conflict in conflicts {
                    println!("   DI {} in {}: {} -> {}", 
                        conflict.di_id, 
                        conflict.protocol,
                        conflict.existing_name,
                        conflict.new_name
                    );
                }
            }
            
            if let Some(errors) = &output.errors {
                println!("\n❌ Errors:");
                for error in errors {
                    println!("   {}", error);
                }
            }
            
            println!("\n🎉 Validation passed! The fix is working correctly.");
            std::process::exit(0);
        }
        Err(e) => {
            println!("❌ FAILED!");
            println!("   Error: {}", e);
            
            let error_msg = format!("{}", e);
            if error_msg.contains("未归类的 .yaml 文件") || error_msg.contains("uncategorized") {
                println!("\n❌ PROTOCOL SUBDIRECTORY BUG STILL EXISTS!");
                println!("   The temporary YAML file is not being placed in the protocol subdirectory.");
            } else {
                println!("\n❓ Different error (might be expected):");
                println!("   {:#?}", e);
            }
            
            std::process::exit(1);
        }
    }
}
