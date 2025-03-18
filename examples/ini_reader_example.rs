use std::error::Error;
use subconverter_rs::utils::ini_reader::IniReader;

fn main() -> Result<(), Box<dyn Error>> {
    // Create example INI content
    let ini_content = r#"
[General]
api_mode=false
api_access_token=password
default_url=
enable_insert=true
update_ruleset_on_request=false

[Proxy]
test=trojan,example.com,443,password,tls-verification=false
test2=vmess,example.org,8080,uuid=00000000-0000-0000-0000-000000000000

[Proxy Group]
Proxy=select,test,test2
"#;

    // Create and parse with the INI reader
    let mut reader = IniReader::new();
    reader.parse(ini_content)?;

    // Get all section names
    println!("Sections: {:?}", reader.get_section_names());

    // Check if specific sections exist
    println!("Has 'General' section: {}", reader.section_exist("General"));
    println!("Has 'Proxy' section: {}", reader.section_exist("Proxy"));

    // Get values from sections
    println!("API mode: {}", reader.get("General", "api_mode"));
    println!("Default URL: {}", reader.get("General", "default_url"));

    // Get as boolean
    println!(
        "Enable insert: {}",
        reader.get_bool("General", "enable_insert")
    );

    // Use current section
    reader.set_current_section("Proxy");
    println!("Test proxy: {}", reader.get_current("test"));

    // Modify values
    reader.set_current("test3", "ss,example.net,8388,password")?;
    println!("Added test3 proxy: {}", reader.get_current("test3"));

    // Export to string
    let exported = reader.to_string();
    println!("\nExported INI:\n{}", exported);

    Ok(())
}
