//! Admin terminal console for SOC attack simulation.
//!
//! Run with:
//!   cargo run --bin admin_terminal -- --base-url http://127.0.0.1:3000
//!
//! It provides a console interface instead of hiding the simulator behind the
//! dashboard UI. It can generate different IPs via X-Forwarded-For so the
//! backend GeoIP/impossible-travel enrichment has deterministic test input.

use anyhow::Context;
use reqwest::Client;
use serde_json::json;
use std::io::{self, Write};
use uuid::Uuid;

const PASSWORD: &str = "P@ssw0rd!12345";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = arg("--base-url").unwrap_or_else(|| std::env::var("BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into()));
    let client = Client::builder().build()?;

    loop {
        println!("\naegis admin terminal");
        println!("1) Generate MFA/login failure alert");
        println!("2) Simulate impossible travel login trail");
        println!("3) Trigger policy-denied alert with bad token");
        println!("4) Run existing pentest battery");
        println!("q) Quit");
        print!("> ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim() {
            "1" => login_failure(&client, &base_url).await?,
            "2" => impossible_travel(&client, &base_url).await?,
            "3" => policy_denied(&client, &base_url).await?,
            "4" => {
                println!("Run: BASE_URL={base_url} cargo run --bin attack_simulator");
            }
            "q" | "quit" | "exit" => break,
            _ => println!("Unknown option"),
        }
    }

    Ok(())
}

fn arg(name: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == name { return args.next(); }
    }
    None
}

async fn login_failure(client: &Client, base_url: &str) -> anyhow::Result<()> {
    let email = format!("sim-{}@example.test", Uuid::new_v4());
    let res = client.post(format!("{base_url}/login"))
        .header("X-Forwarded-For", "8.8.8.8")
        .json(&json!({ "email": email, "password": "wrong" }))
        .send().await?;
    println!("login failure emitted: HTTP {}", res.status());
    Ok(())
}

async fn impossible_travel(client: &Client, base_url: &str) -> anyhow::Result<()> {
    let email = format!("travel-{}@example.test", Uuid::new_v4());
    client.post(format!("{base_url}/register"))
        .header("X-Forwarded-For", "8.8.8.8")
        .json(&json!({ "email": email, "password": PASSWORD }))
        .send().await.context("register")?;

    let first = client.post(format!("{base_url}/login"))
        .header("X-Forwarded-For", "8.8.8.8")
        .json(&json!({ "email": email, "password": PASSWORD }))
        .send().await.context("first login")?;
    println!("first login from US: HTTP {}", first.status());

    let second = client.post(format!("{base_url}/login"))
        .header("X-Forwarded-For", "1.1.1.1")
        .json(&json!({ "email": email, "password": PASSWORD }))
        .send().await.context("second login")?;
    println!("second login from AU: HTTP {}", second.status());
    println!("Dashboard should now show GeoIP data and an impossible-travel alert/event.");
    Ok(())
}

async fn policy_denied(client: &Client, base_url: &str) -> anyhow::Result<()> {
    let res = client.get(format!("{base_url}/admin/dashboard"))
        .bearer_auth("not.a.jwt")
        .send().await?;
    println!("policy/auth denial emitted: HTTP {}", res.status());
    Ok(())
}
