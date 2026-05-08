# Security Considerations

## Summary

As a local CLI application with no network-facing components, the attack surface is minimal. However, several practices could introduce risks if the application is extended or if an attacker gains local access.

---

## Findings

### 1. SQL Injection Risk — Low

**Assessment**: The application uses SQLx's compile-time checked queries with bound parameters. There are no string-concatenated SQL queries.

**Exception**: `get_candidates.rs` dynamically generates a JSON string that is bound as a parameter:
```rust
.bind(&serde_json::to_string(&home_homies.iter()...).expect("..."))
```
While the JSON is generated from typed `i32` values (not user strings), the `expect` panic is suboptimal.

**Fix**: Use `?` operator instead of `expect` to propagate serialization errors safely.

---

### 2. Path Traversal in Config/DB Paths

**Location**: `src/main.rs`, `src/config.rs`

The config file path is constructed as:
```rust
let mut config_file = dirs::home_dir().expect("No home");
config_file.push(".config/local/lunch.json");
```

And the DB URL from config is used directly:
```rust
let database_url = std::env::var("DATABASE_URL").unwrap_or(settings.database_url);
```

**Risk**: If a malicious config file sets `database_url` to `sqlite:/etc/passwd` or another sensitive file, the application will attempt to open it. SQLite's `ATTACH` could theoretically be abused, but the app doesn't execute arbitrary SQL.

**Mitigation**: Validate that the DB path is within the user's home directory or a known-safe location.

```rust
fn validate_db_path(path: &str) -> Result<(), ConfigError> {
    let url = Url::parse(path)?;
    if url.scheme() != "sqlite" {
        return Err(ConfigError::InvalidDatabaseUrl);
    }
    let path = url.to_file_path().map_err(|_| ConfigError::InvalidDatabaseUrl)?;
    let home = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
    if !path.starts_with(&home) {
        return Err(ConfigError::DatabasePathOutsideHome);
    }
    Ok(())
}
```

---

### 3. Telemetry Data Exposure

**Location**: `src/main.rs`

OpenTelemetry is configured to export to `http://localhost:4317`:
```rust
.with_endpoint("http://localhost:4317")
```

**Risk**: If telemetry is enabled on a shared machine or if the local collector is compromised, application behavior data (restaurant picks, homie names) could be exfiltrated.

**Mitigation**:
- Document what data is collected in a privacy policy.
- Allow users to review/audit telemetry payload before sending.
- Hash or anonymize PII (homie names, restaurant names) in traces.

---

### 4. File Permissions

**Risk**: The config file (`~/.config/local/lunch.json`) and DB file (`~/.local/state/lunch.db`) are created with default umask permissions (typically `644`). This means other users on the same system can read the config and potentially the DB.

**Mitigation**: Set restrictive permissions on creation:

```rust
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

let file = OpenOptions::new()
    .write(true)
    .create(true)
    .mode(0o600)  // Owner read/write only
    .open(&config_file)?;
```

---

### 5. Input Validation Gaps

**Location**: `create_restaurant`

Unlike `create_homie`, `create_restaurant` does not validate the input name. While the DB CHECK constraint prevents empty names, the error message is a generic DB error rather than a user-friendly validation message.

**Risk**: Low for a CLI, but inconsistent validation could allow unexpected input if the app is extended.

---

### 6. Dependency Audit

**Wildcards**:
```toml
futures = { version = "*", ... }
```

This accepts any version, including future major versions with breaking changes or security vulnerabilities.

**Fix**: Pin to `futures = "0.3"`.

---

## Recommendations

| Priority | Action |
|----------|--------|
| High | Pin `futures` to a specific version |
| Medium | Validate DB path is within user's home directory |
| Medium | Set `0o600` permissions on config and DB files |
| Low | Document telemetry data collection |
| Low | Add input validation to `create_restaurant` matching `create_homie` |
