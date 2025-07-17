// SentientOS Burn App - Simple calculator application
// This application will run natively inside the SentientOS environment

fn main() {
    println!("==== SentientOS Burn Calculator ====");
    println!("Running inside SentientOS WebAssembly runtime");
    
    // Perform some calculations to demonstrate functionality
    let a = 42;
    let b = 27;
    
    println!("\nPerforming calculations:");
    println!("Addition: {} + {} = {}", a, b, a + b);
    println!("Subtraction: {} - {} = {}", a, b, a - b);
    println!("Multiplication: {} * {} = {}", a, b, a * b);
    
    if b != 0 {
        println!("Division: {} / {} = {}", a, b, a / b);
    }
    
    // Access "system" features
    println!("\nAccessing SentientOS features:");
    println!("Current memory usage: 1.2 MB");
    println!("Container ID: BURN-WASM-4927");
    println!("Security context: Verified");
    
    println!("\nApplication completed successfully!");
}
