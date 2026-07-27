//! Zero Trust Attack Simulator — expanded pentest battery.
//!
//! Exercises the local API and prints PASS/FAIL per attack. PASS = the system
//! defended (attack blocked); FAIL = real finding to investigate. Ends with a
//! summary so it reads like a pentest report.
//!
//! Run the server first (`cargo run`), then `cargo run --bin attack_simulator`.
//! Requires Postgres + Redis up. Uses unique users per run so it is repeatable.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use redis::AsyncCommands;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use uuid::Uuid;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:3000";
const PASSWORD: &str = "StrongPassword123!";

static PASSED: AtomicUsize = AtomicUsize::new(0);
static FAILED: AtomicUsize = AtomicUsize::new(0);

fn verdict(name: &str, defended: bool, detail: &str) {
    if defended {
        PASSED.fetch_add(1, Ordering::Relaxed);
        println!("  \x1b[32mPASS\x1b[0m  {name} — defended ({detail})");
    } else {
        FAILED.fetch_add(1, Ordering::Relaxed);
        println!("  \x1b[31mFAIL\x1b[0m  {name} — ATTACK SUCCEEDED ({detail})");
    }
}

fn unique_email(prefix: &str) -> String {
    format!("{prefix}-{}@example.com", Uuid::new_v4().simple())
}

fn b64url(input: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let client = Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("zt-attack-simulator/2.0")
        .build()?;

    // Subcommand dispatch. Default (no args) runs the one-shot pentest battery;
    // `storm` runs a continuous, tunable adversary to drive the live SOC.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("storm") {
        return run_storm(&client, &base_url, &args).await;
    }

    println!("==========================================================");
    println!("  Zero-Trust Attack Simulator (v2) — pentest battery");
    println!("  target: {base_url}");
    println!("==========================================================");

    print_risk_counters("before").await.ok();

    // Auth surface
    a01_unauth_protected(&client, &base_url).await?;
    a02_rbac_bypass(&client, &base_url).await?;
    a03_garbage_bearer(&client, &base_url).await?;

    // JWT attacks
    a04_alg_none(&client, &base_url).await?;
    a05_signature_strip(&client, &base_url).await?;
    a06_signature_tamper(&client, &base_url).await?;
    a07_payload_tamper_priv_esc(&client, &base_url).await?;
    a08_expired_iat_nbf_skew(&client, &base_url).await?;

    // Session / refresh
    a09_refresh_replay(&client, &base_url).await?;
    a10_logout_then_reuse(&client, &base_url).await?;

    // Input / enumeration
    a11_sql_injection_login(&client, &base_url).await?;
    a12_user_enum_login(&client, &base_url).await?;
    a13_user_enum_forgot(&client, &base_url).await?;
    a14_reset_token_guess(&client, &base_url).await?;

    // Rate limiting
    a15_brute_force_lockout(&client, &base_url).await?;
    a16_edge_rate_limit(&client, &base_url).await?;

    // Header injection / smuggling
    a17_header_injection(&client, &base_url).await?;
    a18_oversized_body(&client, &base_url).await?;

    // Security middleware presence
    a19_security_headers(&client, &base_url).await?;

    // Passkeys safety (login must be disabled until real WebAuthn lands)
    a20_passkey_login_disabled(&client, &base_url).await?;

    print_risk_counters("after").await.ok();

    let passed = PASSED.load(Ordering::Relaxed);
    let failed = FAILED.load(Ordering::Relaxed);
    println!("\n==========================================================");
    println!("  RESULT: {passed} defended, {failed} FAILED");
    println!("==========================================================");
    if failed > 0 {
        println!("  \x1b[31mReview the FAIL lines above — those are real findings.\x1b[0m");
        std::process::exit(1);
    } else {
        println!("  \x1b[32mAll attacks were defended.\x1b[0m");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn fresh_user(client: &Client, base_url: &str) -> anyhow::Result<(String, String, String, String)> {
    let email = unique_email("atk");
    client.post(format!("{base_url}/register"))
        .json(&json!({ "email": email, "password": PASSWORD }))
        .send().await?;
    let res = client.post(format!("{base_url}/login"))
        .json(&json!({ "email": email, "password": PASSWORD }))
        .send().await?;
    let body: Value = res.json().await.unwrap_or_else(|_| json!({}));
    let access = body["data"]["access_token"].as_str().unwrap_or("").to_string();
    let refresh = body["data"]["refresh_token"].as_str().unwrap_or("").to_string();
    let jti = body["data"]["jti"].as_str().unwrap_or("").to_string();
    Ok((email, access, refresh, jti))
}

// ---------------------------------------------------------------------------
// Attacks
// ---------------------------------------------------------------------------

async fn a01_unauth_protected(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A01] Unauthenticated access to /logout");
    let res = client.post(format!("{base_url}/logout")).send().await?;
    verdict("A01", res.status() == StatusCode::UNAUTHORIZED, &format!("status {}", res.status()));
    Ok(())
}

async fn a02_rbac_bypass(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A02] RBAC bypass: normal user -> /admin/dashboard");
    let (_e, access, _, _) = fresh_user(client, base_url).await?;
    let res = client.get(format!("{base_url}/admin/dashboard"))
        .bearer_auth(&access).send().await?;
    verdict("A02", res.status() == StatusCode::UNAUTHORIZED, &format!("status {}", res.status()));
    Ok(())
}

async fn a03_garbage_bearer(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A03] Garbage bearer token");
    let res = client.post(format!("{base_url}/logout"))
        .bearer_auth("not.a.jwt").send().await?;
    verdict("A03", res.status() == StatusCode::UNAUTHORIZED, &format!("status {}", res.status()));
    Ok(())
}

async fn a04_alg_none(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A04] alg:none JWT forgery (classic critical)");
    let header = b64url(b"{\"alg\":\"none\",\"typ\":\"JWT\"}");
    let payload = b64url(
        format!(
            "{{\"sub\":1,\"jti\":\"{}\",\"purpose\":\"access\",\"iat\":0,\"nbf\":0,\"exp\":9999999999,\"iss\":\"auth-service\",\"aud\":\"auth-service-users\"}}",
            Uuid::new_v4()
        ).as_bytes());
    let forged = format!("{header}.{payload}.");
    let res = client.get(format!("{base_url}/admin/dashboard"))
        .bearer_auth(&forged).send().await?;
    verdict("A04 alg:none", res.status() == StatusCode::UNAUTHORIZED,
        &format!("status {} (server enforces RS256)", res.status()));
    Ok(())
}

async fn a05_signature_strip(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A05] Strip JWT signature segment");
    let (_e, access, _, _) = fresh_user(client, base_url).await?;
    let stripped = match access.rsplit_once('.') {
        Some((head, _)) => format!("{head}."),
        None => access.clone(),
    };
    let res = client.post(format!("{base_url}/logout"))
        .bearer_auth(&stripped).send().await?;
    verdict("A05 sig strip", res.status() == StatusCode::UNAUTHORIZED, &format!("status {}", res.status()));
    Ok(())
}

async fn a06_signature_tamper(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A06] Tamper JWT signature");
    let (_e, access, _, _) = fresh_user(client, base_url).await?;
    let tampered = match access.rsplit_once('.') {
        Some((head, _)) => format!("{head}.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
        None => access.clone(),
    };
    let res = client.post(format!("{base_url}/logout"))
        .bearer_auth(&tampered).send().await?;
    verdict("A06 sig tamper", res.status() == StatusCode::UNAUTHORIZED, &format!("status {}", res.status()));
    Ok(())
}

async fn a07_payload_tamper_priv_esc(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A07] Tamper payload (sub->1), keep original signature");
    let (_e, access, _, _) = fresh_user(client, base_url).await?;
    let parts: Vec<&str> = access.split('.').collect();
    let tampered = if parts.len() == 3 {
        let evil = b64url(
            format!(
                "{{\"sub\":1,\"jti\":\"{}\",\"purpose\":\"access\",\"iat\":0,\"nbf\":0,\"exp\":9999999999,\"iss\":\"auth-service\",\"aud\":\"auth-service-users\"}}",
                Uuid::new_v4()
            ).as_bytes());
        format!("{}.{}.{}", parts[0], evil, parts[2])
    } else { access.clone() };
    let res = client.get(format!("{base_url}/admin/dashboard"))
        .bearer_auth(&tampered).send().await?;
    verdict("A07 payload tamper", res.status() == StatusCode::UNAUTHORIZED,
        &format!("status {} (signature breaks)", res.status()));
    Ok(())
}

async fn a08_expired_iat_nbf_skew(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A08] Forged token with very-old exp/iat (clock skew abuse)");
    let header = b64url(b"{\"alg\":\"RS256\",\"typ\":\"JWT\"}");
    // Past exp -> server must reject regardless of signature.
    let payload = b64url(
        format!(
            "{{\"sub\":1,\"jti\":\"{}\",\"purpose\":\"access\",\"iat\":0,\"nbf\":0,\"exp\":1,\"iss\":\"auth-service\",\"aud\":\"auth-service-users\"}}",
            Uuid::new_v4()
        ).as_bytes());
    // Signature is irrelevant — token is expired and the signature is wrong anyway.
    let forged = format!("{header}.{payload}.AAAA");
    let res = client.get(format!("{base_url}/admin/dashboard"))
        .bearer_auth(&forged).send().await?;
    verdict("A08 expired token", res.status() == StatusCode::UNAUTHORIZED, &format!("status {}", res.status()));
    Ok(())
}

async fn a09_refresh_replay(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A09] Refresh-token replay (must revoke family)");
    let (_e, _a, refresh, jti) = fresh_user(client, base_url).await?;
    let first = client.post(format!("{base_url}/refresh"))
        .json(&json!({ "refresh_token": refresh, "jti": jti }))
        .send().await?;
    let first_ok = first.status().is_success();
    let replay = client.post(format!("{base_url}/refresh"))
        .json(&json!({ "refresh_token": refresh, "jti": jti }))
        .send().await?;
    verdict("A09 refresh replay", first_ok && replay.status() == StatusCode::UNAUTHORIZED,
        &format!("first={}, replay={}", if first_ok {"200"} else {"!200"}, replay.status()));
    Ok(())
}

async fn a10_logout_then_reuse(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A10] Reuse access token after logout");
    let (_e, access, _, _) = fresh_user(client, base_url).await?;
    let _ = client.post(format!("{base_url}/logout")).bearer_auth(&access).send().await?;
    let res = client.post(format!("{base_url}/logout")).bearer_auth(&access).send().await?;
    verdict("A10 post-logout reuse", res.status() == StatusCode::UNAUTHORIZED, &format!("status {}", res.status()));
    Ok(())
}

async fn a11_sql_injection_login(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A11] SQL injection in login email");
    let res = client.post(format!("{base_url}/login"))
        .json(&json!({ "email": "' OR '1'='1' --", "password": "x" }))
        .send().await?;
    let blocked = res.status() == StatusCode::UNAUTHORIZED || res.status() == StatusCode::BAD_REQUEST;
    verdict("A11 SQLi", blocked, &format!("status {} (parameterized queries)", res.status()));
    Ok(())
}

async fn a12_user_enum_login(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A12] User enumeration via login error differences");
    let (email, _a, _r, _j) = fresh_user(client, base_url).await?;
    let exists = client.post(format!("{base_url}/login"))
        .json(&json!({ "email": email, "password": "wrong-pw" })).send().await?;
    let exists_code = exists.status();
    let exists_body: Value = exists.json().await.unwrap_or_else(|_| json!({}));
    let nope = client.post(format!("{base_url}/login"))
        .json(&json!({ "email": unique_email("ghost"), "password": "wrong-pw" })).send().await?;
    let nope_code = nope.status();
    let nope_body: Value = nope.json().await.unwrap_or_else(|_| json!({}));
    let same = exists_code == nope_code && exists_body["error"]["code"] == nope_body["error"]["code"];
    verdict("A12 login enum", same, &format!("existing={exists_code}, unknown={nope_code}, identical={same}"));
    Ok(())
}

async fn a13_user_enum_forgot(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A13] User enumeration via /password/forgot");
    let (email, _a, _r, _j) = fresh_user(client, base_url).await?;
    let known = client.post(format!("{base_url}/password/forgot"))
        .json(&json!({ "email": email })).send().await?;
    let known_code = known.status();
    let known_body: Value = known.json().await.unwrap_or_else(|_| json!({}));
    let unknown = client.post(format!("{base_url}/password/forgot"))
        .json(&json!({ "email": unique_email("ghost") })).send().await?;
    let unknown_code = unknown.status();
    let unknown_body: Value = unknown.json().await.unwrap_or_else(|_| json!({}));
    let same = known_code == unknown_code && known_body["data"] == unknown_body["data"];
    verdict("A13 forgot enum", same, &format!("known={known_code}, unknown={unknown_code}, identical={same}"));
    Ok(())
}

async fn a14_reset_token_guess(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A14] Password reset with guessed token");
    let res = client.post(format!("{base_url}/password/reset"))
        .json(&json!({ "token": "made-up-token-1234567890abc", "new_password": "NewPassword999!" }))
        .send().await?;
    let blocked = res.status() == StatusCode::UNAUTHORIZED || res.status() == StatusCode::BAD_REQUEST;
    verdict("A14 reset guess", blocked, &format!("status {}", res.status()));
    Ok(())
}

async fn a15_brute_force_lockout(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A15] Brute-force login -> 429 within 12 attempts");
    let (email, _a, _r, _j) = fresh_user(client, base_url).await?;
    let mut seen = false;
    for i in 1..=12 {
        let res = client.post(format!("{base_url}/login"))
            .json(&json!({ "email": email, "password": format!("wrong-{i}") }))
            .send().await?;
        if res.status() == StatusCode::TOO_MANY_REQUESTS { seen = true; break; }
    }
    verdict("A15 brute lockout", seen, "429 returned");
    Ok(())
}

async fn a16_edge_rate_limit(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A16] Edge rate limit on /register (volumetric)");
    let mut seen = false;
    for i in 1..=60 {
        let res = client.post(format!("{base_url}/register"))
            .json(&json!({ "email": unique_email(&format!("flood-{i}")), "password": PASSWORD }))
            .send().await?;
        if res.status() == StatusCode::TOO_MANY_REQUESTS { seen = true; break; }
    }
    verdict("A16 edge rate limit", seen, "429 within 60 attempts");
    Ok(())
}

async fn a17_header_injection(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A17] Header injection in login email");
    let res = client.post(format!("{base_url}/login"))
        .json(&json!({ "email": "test@example.com\r\nX-Injected: yes", "password": "x" }))
        .send().await?;
    let blocked = res.status().is_client_error();
    verdict("A17 header injection", blocked, &format!("status {} (input rejected)", res.status()));
    Ok(())
}

async fn a18_oversized_body(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A18] Oversized request body (DoS attempt)");
    let huge = "A".repeat(5 * 1024 * 1024); // 5 MiB > 1 MiB cap
    let res = client.post(format!("{base_url}/login"))
        .body(format!("{{\"email\":\"{huge}\",\"password\":\"x\"}}"))
        .header("content-type", "application/json")
        .send().await;
    let defended = match res {
        Ok(r) => r.status() == StatusCode::PAYLOAD_TOO_LARGE || r.status().is_client_error(),
        Err(_) => true, // connection killed = body limit working
    };
    verdict("A18 oversized body", defended, "body limit / payload-too-large");
    Ok(())
}

async fn a19_security_headers(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A19] Security response headers present");
    let res = client.post(format!("{base_url}/login"))
        .json(&json!({ "email": "x@x.x", "password": "x" }))
        .send().await?;
    let h = res.headers();
    let xfo = h.get("x-frame-options").is_some();
    let xcto = h.get("x-content-type-options").is_some();
    let csp = h.get("content-security-policy").is_some();
    let referrer = h.get("referrer-policy").is_some();
    let all = xfo && xcto && csp && referrer;
    verdict("A19 sec headers", all,
        &format!("XFO={xfo} XCTO={xcto} CSP={csp} Referrer={referrer}"));
    Ok(())
}

async fn a20_passkey_login_disabled(client: &Client, base_url: &str) -> anyhow::Result<()> {
    println!("\n[A20] Passkey login endpoints are disabled (until real WebAuthn)");
    let res = client.post(format!("{base_url}/passkeys/login/begin"))
        .json(&json!({ "email": "x@x.x" }))
        .send().await?;
    // 503 = explicitly disabled (good). Anything 2xx = the bypass is live (bad).
    let disabled = res.status() == StatusCode::SERVICE_UNAVAILABLE;
    verdict("A20 passkey disabled", disabled,
        &format!("status {} (expect 503 until WebAuthn signature verification is wired)", res.status()));
    Ok(())
}

// ---------------------------------------------------------------------------
// Telemetry
// ---------------------------------------------------------------------------

async fn print_risk_counters(stage: &str) -> anyhow::Result<()> {
    println!("\n[telemetry:{stage}] Redis lockouts (sample)");
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = redis::Client::open(url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    for key in ["auth:lock:ip:127.0.0.1"] {
        let value: Option<String> = conn.get(key).await.ok().flatten();
        let ttl: i64 = conn.ttl(key).await.unwrap_or(-2);
        println!("  {key} = {} | ttl={ttl}", value.unwrap_or_else(|| "0".into()));
    }
    Ok(())
}

// ===========================================================================
// STORM MODE — continuous adversary for the live SOC demo.
//
//   cargo run --bin attack_simulator storm
//   cargo run --bin attack_simulator storm --rps 6 --secs 180 --victims 8
//
// Fires a weighted mix of attacks at a steady rate against a pool of throwaway
// victim users, so the admin dashboard fills with login failures, lockouts,
// refresh-replay criticals, and RBAC denials in real time. Quiet output: a
// periodic counter line, then a summary. Ctrl-C to stop early.
// ===========================================================================

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn flag_u64(args: &[String], name: &str, default: u64) -> u64 {
    flag(args, name).and_then(|v| v.parse().ok()).unwrap_or(default)
}

async fn run_storm(client: &Client, base_url: &str, args: &[String]) -> anyhow::Result<()> {
    let rps = flag_u64(args, "--rps", 4).max(1);
    let secs = flag_u64(args, "--secs", 60);
    let victim_count = flag_u64(args, "--victims", 5).max(1) as usize;

    println!("==========================================================");
    println!("  Zero-Trust Attack Simulator — STORM MODE (red team)");
    println!("  target: {base_url}");
    println!("  rate: {rps} req/s · duration: {secs}s · victims: {victim_count}");
    println!("  watch the admin SOC dashboard light up. Ctrl-C to stop.");
    println!("==========================================================\n");

    // Provision a pool of victim accounts to attack.
    let mut victims: Vec<String> = Vec::with_capacity(victim_count);
    for _ in 0..victim_count {
        let email = unique_email("victim");
        let _ = client
            .post(format!("{base_url}/register"))
            .json(&json!({ "email": email, "password": PASSWORD }))
            .send()
            .await;
        victims.push(email);
    }
    println!("  provisioned {} victim account(s)\n", victims.len());

    let interval = Duration::from_millis(1000 / rps);
    let deadline = std::time::Instant::now() + Duration::from_secs(secs);

    let mut fired: u64 = 0;
    let mut by_kind: std::collections::BTreeMap<&'static str, u64> = std::collections::BTreeMap::new();
    let mut last_report = std::time::Instant::now();

    while std::time::Instant::now() < deadline {
        let roll = (rand::random::<u32>() % 100) as u32;
        let kind = if roll < 50 {
            let victim = &victims[(rand::random::<u32>() as usize) % victims.len()];
            storm_failed_login(client, base_url, victim).await;
            "failed_login"
        } else if roll < 68 {
            storm_refresh_replay(client, base_url).await;
            "refresh_replay"
        } else if roll < 84 {
            storm_rbac_bypass(client, base_url).await;
            "rbac_denied"
        } else if roll < 94 {
            storm_alg_none(client, base_url).await;
            "alg_none_forgery"
        } else {
            storm_enum_probe(client, base_url).await;
            "enum_probe"
        };
        *by_kind.entry(kind).or_insert(0) += 1;
        fired += 1;

        if last_report.elapsed() >= Duration::from_secs(5) {
            let detail: Vec<String> = by_kind.iter().map(|(k, n)| format!("{k}={n}")).collect();
            println!("  [storm] {fired} attacks fired — {}", detail.join(" "));
            last_report = std::time::Instant::now();
        }

        tokio::time::sleep(interval).await;
    }

    println!("\n==========================================================");
    println!("  STORM COMPLETE — {fired} attacks fired");
    for (k, n) in &by_kind {
        println!("    {k}: {n}");
    }
    println!("  Check the SOC dashboard: event rate, severity mix, alerts.");
    println!("==========================================================");
    Ok(())
}

async fn storm_failed_login(client: &Client, base_url: &str, email: &str) {
    let _ = client
        .post(format!("{base_url}/login"))
        .json(&json!({ "email": email, "password": "wrong-password-storm" }))
        .send()
        .await;
}

async fn storm_refresh_replay(client: &Client, base_url: &str) {
    if let Ok((_e, _a, refresh, jti)) = fresh_user(client, base_url).await {
        // First refresh rotates; replay of the same jti trips family revocation.
        let _ = client
            .post(format!("{base_url}/refresh"))
            .json(&json!({ "refresh_token": refresh, "jti": jti }))
            .send()
            .await;
        let _ = client
            .post(format!("{base_url}/refresh"))
            .json(&json!({ "refresh_token": refresh, "jti": jti }))
            .send()
            .await;
    }
}

async fn storm_rbac_bypass(client: &Client, base_url: &str) {
    if let Ok((_e, access, _r, _j)) = fresh_user(client, base_url).await {
        let _ = client
            .get(format!("{base_url}/admin/dashboard"))
            .bearer_auth(&access)
            .send()
            .await;
    }
}

async fn storm_alg_none(client: &Client, base_url: &str) {
    let header = b64url(b"{\"alg\":\"none\",\"typ\":\"JWT\"}");
    let payload = b64url(
        format!(
            "{{\"sub\":1,\"jti\":\"{}\",\"purpose\":\"access\",\"iat\":0,\"nbf\":0,\"exp\":9999999999,\"iss\":\"auth-service\",\"aud\":\"auth-service-users\"}}",
            Uuid::new_v4()
        )
        .as_bytes(),
    );
    let forged = format!("{header}.{payload}.");
    let _ = client
        .get(format!("{base_url}/admin/dashboard"))
        .bearer_auth(&forged)
        .send()
        .await;
}

async fn storm_enum_probe(client: &Client, base_url: &str) {
    let _ = client
        .post(format!("{base_url}/password/forgot"))
        .json(&json!({ "email": unique_email("ghost") }))
        .send()
        .await;
}
