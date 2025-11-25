# 🔐 EnvLock

Secure, local-first `.env` encryption for developers.

EnvLock encrypts your `.env` files using **Argon2id + XChaCha20-Poly1305** and stores the encrypted output inside a dedicated `.envlock/` folder.  
You can safely commit the encrypted version to Git, sync it between machines, or share it with your team — **without exposing any secrets**.

No cloud. No backend. 100% local.

---

## ✨ Features

- **Local-first encryption** — No cloud dependency
- **Strong cryptography** — Argon2id (KDF) + XChaCha20-Poly1305 (AEAD)
- **`.envlock/` project folder** for clean structure
- **Git-friendly** — only encrypted files are committed
- **Zero dependencies** — just a Rust binary
- **Colorful success & error messages**
- **Profiles supported via custom paths**
- **Safe metadata (`env.meta.json`) stored separately**
- **Config file (`envlock/config.json`) for project defaults**

Planned future features:

- VSCode extension
- Additional sync backends (S3, local folder, SFTP)
- History & snapshots
- Multi-profile system (`dev`, `stage`, `prod`)

---

## 📦 Installation

### Download binary (Releases)

See:
👉 **[https://github.com/harunozceyhan/envlock/releases](https://github.com/harunozceyhan/envlock/releases)**

Binaries provided for:

- macOS (Apple + Intel)
- Linux
- Windows

---

## 🚀 Quick Start

### 1. Initialize project

```bash
envlock init
```

This creates:

```
.envlock/
    config.json
```

Default config:

```json
{
  "env_file": ".env",
  "encrypted_file": ".envlock/.env.enc",
  "meta_file": ".envlock/.env.meta.json"
}
```

---

## 🔐 Commands

Below are all core commands implemented so far.

---

### ### 1. `envlock init`

Initializes the project.

```bash
envlock init
```

Creates:

```
.envlock/config.json
```

---

### ### 2. `envlock lock`

Encrypt your `.env` file.

```bash
envlock lock
```

Or custom paths:

```bash
envlock lock \
  --env .env.local \
  --enc .envlock/local.enc \
  --meta .envlock/local.meta.json \
  --force
```

#### Behavior

- Reads plaintext `.env`
- Asks for password (no echo)
- Derives key (Argon2id)
- Encrypts with XChaCha20-Poly1305
- Writes:

  - encrypted: `.envlock/.env.enc`
  - metadata: `.envlock/.env.meta.json`

---

### ### 3. `envlock unlock`

Decrypt an encrypted `.env` file.

```bash
envlock unlock
```

Or with custom paths:

```bash
envlock unlock --enc .envlock/.env.enc --env .env --meta .envlock/.env.meta.json
```

If `--force` is omitted and `.env` exists, EnvLock will ask for overwrite confirmation.

---

### ### 4. `envlock diff`

Show differences between **plaintext** and **encrypted** env.

```bash
envlock diff
```

Or custom:

```bash
envlock diff --env .env --enc .envlock/dev.enc
```

This command:

- decrypts the encrypted env
- compares key/value pairs
- displays a colored diff

---

### ### 5. `envlock sync`

Encrypt and push encrypted files to Git.

```bash
envlock sync
```

Custom commit message:

```bash
envlock sync --message "Update API keys"
```

Equivalent to:

1. `envlock lock --force`
2. `git add encrypted + meta`
3. `git commit -m <message>`
4. `git push`

---

## 🧠 How It Works (Security Overview)

EnvLock uses:

### ✔ Argon2id

As password-based key derivation (KDF).
Memory-hard → highly resistant to GPU cracking.

### ✔ XChaCha20-Poly1305

Modern authenticated encryption (AEAD).
Prevents tampering and leaking partial plaintext.

### ✔ Metadata file (`env.meta.json`)

Contains:

- salt
- nonce
- Argon2 parameters
- version

**Never contains plaintext or password.**

### ✔ `.envlock/` folder

Contains ONLY encrypted data:

```
.envlock/
    .env.enc
    .env.meta.json
    config.json
```

Plaintext `.env` stays untracked.

---

## 📁 Project Structure (after init)

```
your-project/
  .env                  # plaintext, gitignored
  .envlock/
      .env.enc          # encrypted
      .env.meta.json    # metadata
      config.json       # defaults
```

---

## ❇ Examples

### Encrypt an alternate env file

```bash
envlock lock --env .env.dev --enc .envlock/dev.enc --meta .envlock/dev.meta.json
```

### Decrypt into a disposable file

```bash
envlock unlock --env /tmp/myenv
```

### Compare encrypted file with a new .env

```bash
envlock diff --env .env --enc .envlock/prod.enc
```

---

## 🛠 Build from source

```bash
git clone https://github.com/harunozceyhan/envlock
cd envlock
cargo build --release
```

---

## 💖 Contributing

Pull requests, issues, and feature discussions are welcome!

Current priorities:

- VSCode extension
- Additional sync providers (S3, local folder)
- Multi-profile support
- History snapshots

---

## 📄 License

MIT License.
Commercial-friendly.
Modify and build on top freely.

---

## ⭐ Support the Project

If EnvLock helps you:

- ⭐ Star the repo
- 💬 Share feedback
- 🐛 Report bugs
- 🧑‍💻 Contribute code

Thanks for trying EnvLock!
