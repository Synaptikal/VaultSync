use vaultsync::auth::{hash_password, verify_password};

fn main() {
    println!("Testing Auth Logic...");

    let password = "password123";
    println!("Password: {}", password);

    // 1. Hash
    let hash = match hash_password(password) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Hashing failed: {}", e);
            return;
        }
    };
    println!("Hash: {}", hash);

    // 2. Verify
    match verify_password(password, &hash) {
        Ok(valid) => {
            if valid {
                println!("✅ Verification SUCCESS");
            } else {
                println!("❌ Verification FAILED (returned false)");
            }
        },
        Err(e) => eprintln!("❌ Verification ERROR: {}", e),
    }

    // 3. Verify Wrong Password
    match verify_password("wrongpass", &hash) {
        Ok(valid) => {
            if !valid {
                println!("✅ Wrong password correctly rejected");
            } else {
                println!("❌ Wrong password accepted!");
            }
        },
        Err(e) => eprintln!("❌ Verification ERROR: {}", e),
    }
}
