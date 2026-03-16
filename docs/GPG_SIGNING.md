# GPG Signing for Release Artifacts

This document describes how to set up GPG signing for TuitBot release artifacts.

## Prerequisites

- `gpg` command-line tool installed locally
- A GPG key pair (or create one with `gpg --gen-key`)

## Setup

### 1. Export Your Private Key

Export your private GPG key as an armored file (base64-encoded):

```bash
gpg --export-secret-keys --armor YOUR_KEY_ID > private-key.asc
```

### 2. Add Secrets to GitHub

Add the following secrets to the repository (Settings → Secrets and variables → Actions):

1. **`GPG_PRIVATE_KEY`**: Paste the content of `private-key.asc` (the full armored private key)
2. **`GPG_PASSPHRASE`**: The passphrase for your private key (if key is passphrase-protected)

### 3. Verify in Release Workflow

The release workflow (`.github/workflows/release.yml`) will:
1. Import the GPG private key from `GPG_PRIVATE_KEY` secret
2. Sign each release artifact with a detached signature (`.asc` file)
3. Upload both artifacts and signatures to GitHub Releases

### 4. Verify Signatures

Users can verify downloaded artifacts:

```bash
# Download the artifact and its signature
wget https://github.com/aramirez087/TuitBot/releases/download/v1.0.0/tuitbot-cli-v1.0.0-x86_64-unknown-linux-gnu
wget https://github.com/aramirez087/TuitBot/releases/download/v1.0.0/tuitbot-cli-v1.0.0-x86_64-unknown-linux-gnu.asc

# Import the maintainer's public key (if not already in keyring)
gpg --import maintainer-public-key.asc  # Or: gpg --recv-keys KEY_ID

# Verify the signature
gpg --verify tuitbot-cli-v1.0.0-x86_64-unknown-linux-gnu.asc tuitbot-cli-v1.0.0-x86_64-unknown-linux-gnu
```

If the signature is valid, you'll see: `Good signature from "..."`

## Optional: Signed Git Commits

To also sign git commits during releases, configure git locally:

```bash
git config --global user.signingkey YOUR_KEY_ID
git config --global commit.gpgSign true
```

The release workflow can use these settings to sign automated commits (not currently enabled, but the infrastructure is in place).

## References

- [GPG Documentation](https://gnupg.org/gph/en/manual/x110.html)
- [GitHub: Signing commits](https://docs.github.com/en/authentication/managing-commit-signature-verification)
- [CycloneDX / SBOM for supply chain security](https://cyclonedx.org/)
