// SentientOS Package Manager - Java Package Handler
// Handles Java packages using Maven and Gradle

use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use crate::core::constants;

/// Install a Java package
pub fn install_package(name: &str, version: Option<&str>) -> Result<()> {
    info!("Installing Java package: {}", name);
    
    // Check if Java is installed
    let java_check = Command::new("which")
        .arg("java")
        .output()?;
        
    if !java_check.status.success() {
        return Err(anyhow::anyhow!("Java not found, please install JDK"));
    }
    
    // Create packages directory
    let java_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("java");
    fs::create_dir_all(&java_dir)?;
    
    // Determine if the package uses Maven format (groupId:artifactId)
    if name.contains(":") {
        install_maven_package(&java_dir, name, version)?;
    } else {
        install_jar_package(&java_dir, name, version)?;
    }
    
    info!("Java package {} installed successfully", name);
    Ok(())
}

/// Install a Maven package
fn install_maven_package(java_dir: &PathBuf, name: &str, version: Option<&str>) -> Result<()> {
    // Check if Maven is installed
    let maven_check = Command::new("which")
        .arg("mvn")
        .output()?;
        
    if !maven_check.status.success() {
        return Err(anyhow::anyhow!("Maven not found, please install Maven"));
    }
    
    // Create a temporary pom file
    let pom_dir = java_dir.join("maven");
    fs::create_dir_all(&pom_dir)?;
    
    let parts: Vec<&str> = name.split(":").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid Maven package format. Use groupId:artifactId"));
    }
    
    let group_id = parts[0];
    let artifact_id = parts[1];
    let version_str = version.unwrap_or("LATEST");
    
    // Create a minimal POM file
    let pom_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0" 
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" 
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>org.sentientos.wrapper</groupId>
    <artifactId>maven-wrapper</artifactId>
    <version>1.0-SNAPSHOT</version>
    <dependencies>
        <dependency>
            <groupId>{}</groupId>
            <artifactId>{}</artifactId>
            <version>{}</version>
        </dependency>
    </dependencies>
    <repositories>
        <repository>
            <id>central</id>
            <url>https://repo.maven.apache.org/maven2</url>
        </repository>
    </repositories>
</project>"#, group_id, artifact_id, version_str);

    let pom_path = pom_dir.join("pom.xml");
    fs::write(&pom_path, pom_content)?;
    
    // Run Maven to download the dependency
    let mut cmd = Command::new("mvn");
    cmd.current_dir(&pom_dir);
    cmd.args(["dependency:copy-dependencies", "-DoutputDirectory=./lib"]);
    
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to install Maven package: {}\n{}", name, stderr));
    }
    
    Ok(())
}

/// Install a JAR package (direct download)
fn install_jar_package(java_dir: &PathBuf, name: &str, version: Option<&str>) -> Result<()> {
    // For direct JAR downloads, we would typically download from a URL
    // Since this is a simulation, we'll just create a placeholder JAR file
    
    let jars_dir = java_dir.join("jars");
    fs::create_dir_all(&jars_dir)?;
    
    let version_str = version.unwrap_or("latest");
    let jar_name = format!("{}-{}.jar", name, version_str);
    let jar_path = jars_dir.join(&jar_name);
    
    // Create an empty JAR file (in a real implementation, we would download it)
    info!("Creating placeholder JAR file: {}", jar_name);
    fs::write(&jar_path, "Placeholder JAR file")?;
    
    Ok(())
}

/// Remove a Java package
pub fn remove_package(name: &str) -> Result<()> {
    info!("Removing Java package: {}", name);
    
    let java_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("java");
    
    if name.contains(":") {
        // Maven package
        let parts: Vec<&str> = name.split(":").collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid Maven package format. Use groupId:artifactId"));
        }
        
        let artifact_id = parts[1];
        
        // Remove Maven dependencies
        let maven_lib_dir = java_dir.join("maven").join("lib");
        if maven_lib_dir.exists() {
            for entry in fs::read_dir(maven_lib_dir)? {
                let entry = entry?;
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&format!("{}-", artifact_id)) {
                    fs::remove_file(entry.path())?;
                    info!("Removed Maven artifact: {}", file_name);
                }
            }
        }
    } else {
        // JAR package
        let jars_dir = java_dir.join("jars");
        if jars_dir.exists() {
            for entry in fs::read_dir(jars_dir)? {
                let entry = entry?;
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&format!("{}-", name)) {
                    fs::remove_file(entry.path())?;
                    info!("Removed JAR file: {}", file_name);
                }
            }
        }
    }
    
    info!("Java package {} removed successfully", name);
    Ok(())
}

/// Run a Java package with arguments
pub fn run_package(name: &str, args: &[&str]) -> Result<()> {
    info!("Running Java package: {}", name);
    
    // Check if Java is installed
    let java_check = Command::new("which")
        .arg("java")
        .output()?;
        
    if !java_check.status.success() {
        return Err(anyhow::anyhow!("Java not found, please install JDK"));
    }
    
    let java_dir = PathBuf::from(constants::ROOT_DIR).join("packages").join("java");
    
    if name.contains(":") {
        // Maven package
        let parts: Vec<&str> = name.split(":").collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid Maven package format. Use groupId:artifactId"));
        }
        
        let artifact_id = parts[1];
        
        // Find the JAR in the Maven repository
        let maven_lib_dir = java_dir.join("maven").join("lib");
        if !maven_lib_dir.exists() {
            return Err(anyhow::anyhow!("Maven library directory not found"));
        }
        
        let mut jar_path = None;
        for entry in fs::read_dir(maven_lib_dir)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with(&format!("{}-", artifact_id)) && file_name.ends_with(".jar") {
                jar_path = Some(entry.path());
                break;
            }
        }
        
        if let Some(path) = jar_path {
            // Run the JAR file
            let mut cmd = Command::new("java");
            cmd.arg("-jar");
            cmd.arg(path);
            cmd.args(args);
            
            let mut child = cmd.spawn()?;
            let status = child.wait()?;
            
            if !status.success() {
                return Err(anyhow::anyhow!("Java application failed with exit code: {:?}", status.code()));
            }
        } else {
            return Err(anyhow::anyhow!("JAR file not found for package: {}", name));
        }
    } else {
        // Direct JAR package
        let jars_dir = java_dir.join("jars");
        let mut jar_path = None;
        
        if jars_dir.exists() {
            for entry in fs::read_dir(jars_dir)? {
                let entry = entry?;
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&format!("{}-", name)) && file_name.ends_with(".jar") {
                    jar_path = Some(entry.path());
                    break;
                }
            }
        }
        
        if let Some(path) = jar_path {
            // Run the JAR file
            let mut cmd = Command::new("java");
            cmd.arg("-jar");
            cmd.arg(path);
            cmd.args(args);
            
            let mut child = cmd.spawn()?;
            let status = child.wait()?;
            
            if !status.success() {
                return Err(anyhow::anyhow!("Java application failed with exit code: {:?}", status.code()));
            }
        } else {
            return Err(anyhow::anyhow!("JAR file not found for package: {}", name));
        }
    }
    
    Ok(())
}

/// Search for Java packages
pub fn search_packages(query: &str) -> Result<Vec<String>> {
    info!("Searching for Java packages matching: {}", query);
    
    let mut results = Vec::new();
    
    // In a real implementation, we would query Maven Central or other repositories
    // For this prototype, we'll return some simulated results
    
    // Simulate Maven Central results
    if query.len() > 2 {
        results.push(format!("com.example:{} (java) - Java library", query));
        results.push(format!("org.{}.core:core (java) - Core library", query));
        results.push(format!("io.{}:utils (java) - Utility library", query));
    }
    
    Ok(results)
}
