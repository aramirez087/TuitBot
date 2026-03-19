# GPG Signing for Release Artifacts

This document describes how to set up GPG signing for TuitBot release artifacts for supply chain security (Roadmap §4.4).

## Overview

All TuitBot release artifacts are cryptographically signed using GPG. Signature files (`.sig`) are published alongside binaries and checksums to enable verification of authenticity and integrity.

**Signature Format**: Armored ASCII (`.sig` files = detached GPG signatures in text format)

## Prerequisites

- `gpg` command-line tool installed locally (GNU Privacy Guard)
- A GPG key pair for signing (created or imported)

## CI/CD Setup (Maintainers)

### 1. Generate or Import a GPG Key

Create a new signing key for release automation:

```bash
gpg --gen-key
# Follow prompts:
# - Kind: RSA (default)
# - Keysize: 4096 bits (recommended)
# - Validity: 2 years or longer
# - Real name: "TuitBot Release Bot" (or your name)
# - Email: your-email@example.com
# - Passphrase: strong, random passphrase
```

Or import an existing key:

```bash
gpg --import /path/to/private-key.asc
```

### 2. Export the Private Key

Export your private GPG key as armored ASCII (base64-encoded):

```bash
gpg --export-secret-keys --armor YOUR_KEY_ID > private-key.asc
cat private-key.asc
```

### 3. Add GitHub Action Secrets

Add the following secrets to the repository (Settings → Secrets and variables → Actions):

| Secret Name | Value | Notes |
|-------------|-------|-------|
| `GPG_PRIVATE_KEY` | Full content of `private-key.asc` | Entire armored private key block |
| `GPG_PASSPHRASE` | Your GPG key passphrase | Required to unlock the private key during signing |

### 4. Publish Your Public Key

Publish your public GPG key to a keyserver for user verification:

```bash
# Export public key
gpg --export --armor YOUR_KEY_ID > public-key.asc

# Upload to Ubuntu keyserver (most widely used)
gpg --send-keys --keyserver keyserver.ubuntu.com YOUR_KEY_ID

# Or add to your GitHub profile:
# https://github.com/settings/keys → Add GPG key
```

Store the public key in the repository for offline verification:

```bash
cp public-key.asc .github/keys/release-signing-key.pub.asc
git add .github/keys/release-signing-key.pub.asc
git commit -m "chore: add release signing public key"
```

## Signature Workflow

The release workflow (`.github/workflows/release.yml`) automatically:

1. **Import the GPG private key** from the `GPG_PRIVATE_KEY` secret
2. **Build release artifacts** (binaries, archives, SBOM)
3. **Sign each artifact** with a detached signature:
   - Input: `tuitbot-x86_64-unknown-linux-gnu.tar.gz`
   - Output: `tuitbot-x86_64-unknown-linux-gnu.tar.gz.sig` (GPG detached signature)
4. **Upload to GitHub Releases** alongside binaries and checksums
5. **Non-blocking on error**: Signing failures do NOT block release publication (graceful degradation)

## User: Verifying Signatures

### Quick Start (Automated)

```bash
# Download the release artifacts
VERSION="tuitbot-cli-v1.0.0"
TARGET="x86_64-unknown-linux-gnu"
RELEASE_URL="https://github.com/aramirez087/TuitBot/releases/download"

wget "${RELEASE_URL}/${VERSION}/${VERSION}-${TARGET}.tar.gz"
wget "${RELEASE_URL}/${VERSION}/${VERSION}-${TARGET}.tar.gz.sig"

# Verify signature (requires maintainer's public key in your keyring)
gpg --verify "${VERSION}-${TARGET}.tar.gz.sig" "${VERSION}-${TARGET}.tar.gz"
```

Expected output on success:
```
gpg: Signature made Wed Mar 19 16:00:00 2026 UTC
gpg: using RSA key XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
gpg: Good signature from "TuitBot Release Bot <bot@example.com>"
```

### Step-by-Step (Manual)

#### Step 1: Import the Maintainer's Public Key

**Option A**: Download from GitHub release:
```bash
wget https://github.com/aramirez087/TuitBot/releases/download/v1.0.0/.github/keys/release-signing-key.pub.asc
gpg --import release-signing-key.pub.asc
```

**Option B**: Fetch from Ubuntu keyserver:
```bash
gpg --recv-keys MAINTAINER_KEY_ID  # Replace with actual key ID
```

**Option C**: From GitHub profile:
```bash
# Copy the GPG public key from the maintainer's GitHub profile
# and save to signing-key.asc, then:
gpg --import signing-key.asc
```

#### Step 2: Download Artifact and Signature

```bash
cd /tmp/releases
wget https://github.com/aramirez087/TuitBot/releases/download/tuitbot-cli-v1.0.0/tuitbot-cli-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
wget https://github.com/aramirez087/TuitBot/releases/download/tuitbot-cli-v1.0.0/tuitbot-cli-v1.0.0-x86_64-unknown-linux-gnu.tar.gz.sig
```

#### Step 3: Verify the Signature

```bash
gpg --verify tuitbot-cli-v1.0.0-x86_64-unknown-linux-gnu.tar.gz.sig \
    tuitbot-cli-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
```

Success looks like:
```
gpg: Signature made [date/time]
gpg: using RSA key 1234567890ABCDEF
gpg: Good signature from "TuitBot Release Bot <bot@example.com>" [unknown]
gpg: WARNING: This key is not certified with a trusted signature!
```

⚠️ The "key is not certified" warning is normal if you haven't explicitly trusted the key—GPG is just warning you that you haven't validated it through the web-of-trust system. The "Good signature" message confirms the artifact hasn't been tampered with.

#### Step 4: Verify Checksums (Additional Layer)

```bash
# Download checksums
wget https://github.com/aramirez087/TuitBot/releases/download/tuitbot-cli-v1.0.0/SHA256SUMS
wget https://github.com/aramirez087/TuitBot/releases/download/tuitbot-cli-v1.0.0/SHA256SUMS.sig

# Verify the checksum file itself was signed
gpg --verify SHA256SUMS.sig SHA256SUMS

# Verify artifact matches checksum
sha256sum -c SHA256SUMS --ignore-missing
```

## Troubleshooting

### "Unknown public key" Error

```
gpg: No public key
```

**Solution**: Import the maintainer's public key (see Step 1 above).

### "Bad signature" Error

```
gpg: Bad signature from "..."
```

**Causes**:
- Artifact was corrupted during download
- Signature file doesn't match the binary
- Wrong GPG key was imported

**Solution**:
1. Re-download both the artifact and signature
2. Verify you imported the correct public key
3. Check the release tag to ensure you're on the right version

### GPG Key Not Found Locally

```
gpg: keyid 1234567890 not found
```

Fetch the key from a keyserver:

```bash
gpg --recv-keys 1234567890ABCDEF
```

## Release Notes Template

When publishing a release, include signature verification instructions:

```markdown
## ✅ Verify Artifacts

All binaries and archives are signed with GPG. To verify:

1. Download the `.sig` file alongside the binary
2. Import the release signing key (if not already imported):
   \`\`\`bash
   gpg --recv-keys SIGNING_KEY_ID
   \`\`\`
3. Verify the signature:
   \`\`\`bash
   gpg --verify tuitbot-x86_64-unknown-linux-gnu.tar.gz.sig tuitbot-x86_64-unknown-linux-gnu.tar.gz
   \`\`\`

**Expected output**: `gpg: Good signature from "TuitBot Release Bot ..."`

[More details](./docs/GPG_SIGNING.md)
```

## References

- [GNU Privacy Guard (GPG) Manual](https://gnupg.org/gph/en/manual/x110.html)
- [GitHub: GPG Commit Signing](https://docs.github.com/en/authentication/managing-commit-signature-verification)
- [CISA: Software Supply Chain Security](https://www.cisa.gov/software-supply-chain-security)
- [CycloneDX (SBOM)](https://cyclonedx.org/) — paired with GPG signatures for complete supply chain security
