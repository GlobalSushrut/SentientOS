// Sentinent OS CLI - ZK Contract Management
// Provides CLI commands for managing ZK contracts

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::path::{Path, PathBuf};
use std::fs;
use clap::{Arg, ArgMatches, Command, SubCommand};
use colored::Colorize;
use chrono::{DateTime, Utc};

use crate::zk::contracts::ZkContract;
use crate::zk::parser;
use crate::zk::verification;
use crate::zk::executor;
use crate::core::constants;

/// Register ZK subcommand to CLI
pub fn register_commands() -> Command<'static> {
    Command::new("zk")
        .about("Manage zero-knowledge contracts and verification")
        .subcommand(
            Command::new("verify")
                .about("Verify ZK proof integrity")
                .arg(
                    Arg::new("contract")
                        .help("Contract name to verify")
                        .required(true)
                )
                .arg(
                    Arg::new("proof")
                        .help("Proof hash to verify")
                        .required(false)
                )
        )
        .subcommand(
            Command::new("list")
                .about("List ZK contracts")
                .arg(
                    Arg::new("verified")
                        .long("verified")
                        .help("Only show verified contracts")
                        .required(false)
                        .takes_value(false)
                )
        )
        .subcommand(
            Command::new("create")
                .about("Create a new ZK contract")
                .arg(
                    Arg::new("name")
                        .help("Name for the new contract")
                        .required(true)
                )
                .arg(
                    Arg::new("template")
                        .long("template")
                        .short('t')
                        .help("Template to use (basic, storage, auth)")
                        .default_value("basic")
                )
        )
        .subcommand(
            Command::new("run")
                .about("Run a method in a ZK contract")
                .arg(
                    Arg::new("contract")
                        .help("Contract name")
                        .required(true)
                )
                .arg(
                    Arg::new("method")
                        .help("Method to run")
                        .required(true)
                )
                .arg(
                    Arg::new("args")
                        .help("Arguments as JSON array")
                        .required(false)
                        .default_value("[]")
                )
        )
}

/// Handle ZK subcommands
pub fn handle_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("verify", sub_matches)) => {
            cmd_verify(
                sub_matches.get_one::<String>("contract").unwrap(),
                sub_matches.get_one::<String>("proof"),
            )
        },
        Some(("list", sub_matches)) => {
            cmd_list(sub_matches.is_present("verified"))
        },
        Some(("create", sub_matches)) => {
            cmd_create(
                sub_matches.get_one::<String>("name").unwrap(),
                sub_matches.get_one::<String>("template").unwrap(),
            )
        },
        Some(("run", sub_matches)) => {
            cmd_run(
                sub_matches.get_one::<String>("contract").unwrap(),
                sub_matches.get_one::<String>("method").unwrap(),
                sub_matches.get_one::<String>("args").unwrap(),
            )
        },
        _ => {
            println!("{}", "Unknown ZK subcommand".red());
            Ok(())
        }
    }
}

/// Verify a ZK contract proof
fn cmd_verify(contract_name: &str, proof_opt: Option<&String>) -> Result<()> {
    println!("\n{} {} {}\n", "🔐".green(), "Verifying ZK contract:".bold(), contract_name.cyan().bold());
    
    // Check if contract exists
    let zk_dir = PathBuf::from(constants::ROOT_DIR).join(".zk");
    let contracts_dir = zk_dir.join("contracts");
    let contract_file = contracts_dir.join(format!("{}.yaml", contract_name));
    
    if !contract_file.exists() {
        println!("{} {} {}", "❌".red(), "Contract not found:".bold(), contract_name);
        return Ok(());
    }
    
    // If a specific proof was provided
    if let Some(proof) = proof_opt {
        // Verify the specific proof
        match verification::verify_proof(contract_name, proof, "") {
            Ok(result) => {
                match result.status {
                    verification::VerificationStatus::Verified => {
                        println!("{} {} {}", "✅".green(), "Proof verified successfully:".bold(), proof);
                        println!("  {} {}", "Time:".bold(), format_timestamp(result.timestamp));
                    },
                    verification::VerificationStatus::Failed => {
                        println!("{} {} {}", "❌".red(), "Proof verification failed:".bold(), proof);
                        if let Some(error) = result.error {
                            println!("  {} {}", "Error:".bold(), error.red());
                        }
                    },
                    verification::VerificationStatus::NotVerified => {
                        println!("{} {} {}", "⚠️".yellow(), "Proof not verified:".bold(), proof);
                    }
                }
            },
            Err(err) => {
                println!("{} {} {}", "❌".red(), "Verification error:".bold(), err);
            }
        }
    } else {
        // Check if contract is verified
        match verification::is_contract_verified(contract_name) {
            Ok(verified) => {
                if verified {
                    println!("{} {} {}", "✅".green(), "Contract verified:".bold(), contract_name);
                    
                    // Get latest verification result
                    if let Ok(Some(result)) = verification::get_latest_verification(contract_name) {
                        println!("  {} {}", "Last verified:".bold(), format_timestamp(result.timestamp));
                        println!("  {} {}", "Proof:".bold(), result.hash);
                    }
                } else {
                    println!("{} {} {}", "⚠️".yellow(), "Contract not verified:".bold(), contract_name);
                }
                
                // List all verification results
                match verification::list_verification_results(contract_name) {
                    Ok(results) => {
                        if !results.is_empty() {
                            println!("\n{}", "Verification history:".bold());
                            for (i, result) in results.iter().enumerate() {
                                let status_icon = match result.status {
                                    verification::VerificationStatus::Verified => "✅".green(),
                                    verification::VerificationStatus::Failed => "❌".red(),
                                    verification::VerificationStatus::NotVerified => "⚠️".yellow(),
                                };
                                println!("  {}. {} {} ({})", 
                                    i + 1, 
                                    status_icon, 
                                    result.hash,
                                    format_timestamp(result.timestamp)
                                );
                            }
                        }
                    },
                    Err(err) => {
                        println!("{} {} {}", "❌".red(), "Error listing verification results:".bold(), err);
                    }
                }
            },
            Err(err) => {
                println!("{} {} {}", "❌".red(), "Error checking verification:".bold(), err);
            }
        }
    }
    
    Ok(())
}

/// List all ZK contracts
fn cmd_list(verified_only: bool) -> Result<()> {
    println!("\n{} {}\n", "📋".green(), "ZK Contracts".bold());
    
    let zk_dir = PathBuf::from(constants::ROOT_DIR).join(".zk");
    let contracts_dir = zk_dir.join("contracts");
    
    if !contracts_dir.exists() {
        println!("No contracts directory found. Create one at: {}", contracts_dir.display());
        return Ok(());
    }
    
    let mut found_contracts = false;
    
    for entry in fs::read_dir(contracts_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
            let contract_name = path.file_stem().unwrap().to_string_lossy();
            
            // Check if the contract is verified (if filter is enabled)
            let verified = verification::is_contract_verified(&contract_name)?;
            
            if !verified_only || (verified_only && verified) {
                found_contracts = true;
                
                let verification_status = if verified {
                    "✅".green()
                } else {
                    "⚠️".yellow()
                };
                
                // Load the contract to get more details
                let contract_yaml = fs::read_to_string(&path)?;
                if let Ok(contract) = parser::parse_zk_yaml(&contract_yaml) {
                    println!("{} {} (v{})", verification_status, contract_name.cyan().bold(), contract.version);
                    println!("  Methods: {}", contract.methods.keys().cloned().collect::<Vec<_>>().join(", "));
                    println!("  Rules: {}", contract.rules.len());
                    println!();
                } else {
                    println!("{} {} (parse error)", verification_status, contract_name.cyan().bold());
                    println!();
                }
            }
        }
    }
    
    if !found_contracts {
        if verified_only {
            println!("No verified contracts found.");
        } else {
            println!("No contracts found.");
        }
    }
    
    Ok(())
}

/// Create a new ZK contract
fn cmd_create(name: &str, template: &str) -> Result<()> {
    println!("\n{} {} {} ({})\n", "🔨".green(), "Creating ZK contract:".bold(), name.cyan().bold(), template);
    
    let zk_dir = PathBuf::from(constants::ROOT_DIR).join(".zk");
    let contracts_dir = zk_dir.join("contracts");
    
    // Create the contracts directory if it doesn't exist
    fs::create_dir_all(&contracts_dir)?;
    
    let contract_file = contracts_dir.join(format!("{}.yaml", name));
    
    // Check if the contract already exists
    if contract_file.exists() {
        println!("{} {} {}", "❌".red(), "Contract already exists:".bold(), name);
        return Ok(());
    }
    
    // Create contract content based on template
    let contract_content = match template {
        "basic" => format!(r#"
name: {}
version: 0.1.0
state:
  counter: 0
  last_updated: ""
methods:
  increment:
    name: increment
    implementation: |
      // Increment the counter
      state.counter += 1;
      state.last_updated = new Date().toISOString();
      verify_rule("counter_positive");
      return state.counter;
  get_counter:
    name: get_counter
    implementation: |
      // Get the current counter value
      return state.counter;
rules:
  - name: counter_positive
    condition: state.counter >= 0
    effect: revert if counter becomes negative
"#, name),
        "storage" => format!(r#"
name: {}
version: 0.1.0
state:
  storage: {{}}
  owners: []
methods:
  store:
    name: store
    implementation: |
      // Store a value with a key
      const key = args[0];
      const value = args[1];
      state.storage[key] = value;
      verify_rule("valid_storage");
      return true;
  retrieve:
    name: retrieve
    implementation: |
      // Retrieve a value by key
      const key = args[0];
      return state.storage[key] || null;
  add_owner:
    name: add_owner
    implementation: |
      // Add a new owner
      const owner = args[0];
      if (!state.owners.includes(owner)) {{
        state.owners.push(owner);
      }}
      return state.owners;
rules:
  - name: valid_storage
    condition: Object.keys(state.storage).length < 1000
    effect: prevent storage overflow
"#, name),
        "auth" => format!(r#"
name: {}
version: 0.1.0
state:
  users: {{}}
  admin: ""
methods:
  register:
    name: register
    implementation: |
      // Register a new user
      const username = args[0];
      const passwordHash = args[1];
      
      if (state.users[username]) {{
        return false; // User already exists
      }}
      
      state.users[username] = {{
        passwordHash,
        createdAt: new Date().toISOString(),
        isActive: true
      }};
      
      verify_rule("max_users");
      return true;
  authenticate:
    name: authenticate
    implementation: |
      // Authenticate a user
      const username = args[0];
      const passwordHash = args[1];
      
      if (!state.users[username]) {{
        return false; // User does not exist
      }}
      
      return state.users[username].passwordHash === passwordHash &&
             state.users[username].isActive;
  set_admin:
    name: set_admin
    implementation: |
      // Set the admin user
      const username = args[0];
      
      if (!state.users[username]) {{
        return false; // User does not exist
      }}
      
      state.admin = username;
      return true;
rules:
  - name: max_users
    condition: Object.keys(state.users).length < 100
    effect: prevent too many users
"#, name),
        _ => {
            println!("{} {} {}", "❌".red(), "Unknown template:".bold(), template);
            return Ok(());
        }
    };
    
    // Write the contract file
    fs::write(&contract_file, contract_content)?;
    
    println!("{} {} {}", "✅".green(), "Created contract:".bold(), name);
    println!("  {} {}", "Path:".bold(), contract_file.display());
    
    // Attempt to parse and validate the contract
    match fs::read_to_string(&contract_file) {
        Ok(yaml) => match parser::parse_zk_yaml(&yaml) {
            Ok(_) => println!("{} {}", "✅".green(), "Contract validates successfully"),
            Err(err) => println!("{} {} {}", "⚠️".yellow(), "Contract validation failed:".bold(), err),
        },
        Err(err) => println!("{} {} {}", "❌".red(), "Error reading contract:".bold(), err),
    }
    
    Ok(())
}

/// Run a method in a ZK contract
fn cmd_run(contract_name: &str, method_name: &str, args_json: &str) -> Result<()> {
    println!("\n{} {} {} {}\n", "▶️".green(), "Running ZK contract method:".bold(), 
             contract_name.cyan().bold(), method_name.cyan());
    
    // Parse arguments
    let args: Vec<serde_json::Value> = match serde_json::from_str(args_json) {
        Ok(args) => args,
        Err(err) => {
            println!("{} {} {}", "❌".red(), "Invalid JSON arguments:".bold(), err);
            return Ok(());
        }
    };
    
    // Load the contract
    let zk_dir = PathBuf::from(constants::ROOT_DIR).join(".zk");
    let contracts_dir = zk_dir.join("contracts");
    let contract_file = contracts_dir.join(format!("{}.yaml", contract_name));
    
    if !contract_file.exists() {
        println!("{} {} {}", "❌".red(), "Contract not found:".bold(), contract_name);
        return Ok(());
    }
    
    let contract_yaml = fs::read_to_string(&contract_file)?;
    let contract = match parser::parse_zk_yaml(&contract_yaml) {
        Ok(contract) => contract,
        Err(err) => {
            println!("{} {} {}", "❌".red(), "Error parsing contract:".bold(), err);
            return Ok(());
        }
    };
    
    // Check if the method exists
    if !contract.methods.contains_key(method_name) {
        println!("{} {} {}", "❌".red(), "Method not found:".bold(), method_name);
        println!("Available methods: {}", contract.methods.keys().cloned().collect::<Vec<_>>().join(", "));
        return Ok(());
    }
    
    // Execute the method
    match executor::execute_contract_method(&contract, method_name, &args) {
        Ok(result) => {
            println!("{} {}", "Result:".bold(), serde_json::to_string_pretty(&result)?);
            
            // Generate and store a proof of execution
            let input_data = serde_json::to_string(&args)?;
            match verification::generate_proof(&contract, &input_data) {
                Ok(proof) => {
                    println!("{} {}", "Proof:".bold(), proof);
                    println!("{} {}", "✅".green(), "Method executed successfully with proof generation");
                },
                Err(err) => {
                    println!("{} {}", "Method executed successfully but proof generation failed:".bold(), err);
                }
            }
        },
        Err(err) => {
            println!("{} {} {}", "❌".red(), "Error executing method:".bold(), err);
        }
    }
    
    Ok(())
}

/// Format a timestamp for display
fn format_timestamp(timestamp: u64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp as i64, 0)
        .unwrap_or_default();
    
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
