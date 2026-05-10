# XDUpdate

A new generation update verification system.

## Functionality

### Updater

- File integrity verification (SHA-512)
- File signing with a quantum-resistant algoritm (ML-DSA 87)
- A simple wrapper system, which makes it possible to use this program on all leading desktop platforms
- Certificate pinning for prevention of advanced MITM attacks
- TLS is always used

### Update packager

- Generating keypairs required for signing the update files
- Signing the files with the xdupdate header
