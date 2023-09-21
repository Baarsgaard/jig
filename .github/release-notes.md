## Features
-
-

## Changes
-
-

## Fixes
-
-

All changes: https://github.com/baarsgaard/jig/compare/v0.2.0...v${CRATE_VERSION}

### Upgrading:

For the newest release:
```bash
jig upgrade
```


<details>
<summary><h2>Installation</h2></summary>

Pick between cloud and server (APIs differ)

### Linux

```bash
# Cloud
wget -O ~/.local/bin/jig "https://github.com/Raunow/jig/releases/download/v${CRATE_VERSION}/jig-cloud-x86_64-unknown-linux-gnu"
# Server
wget -O ~/.local/bin/jig "https://github.com/Raunow/jig/releases/download/v${CRATE_VERSION}/jig-server-x86_64-unknown-linux-gnu"

chmod +x ~/.local/bin/jig
```

### Windows

```posh
# cloud
Invoke-WebRequest -Uri "https://github.com/Raunow/jig/releases/download/v${CRATE_VERSION}/jig-cloud-x86_64-pc-windows-msvc.exe" -OutFile "C:\<Somewhere in PATH>"
# server
Invoke-WebRequest -Uri "https://github.com/Raunow/jig/releases/download/v${CRATE_VERSION}/jig-server-x86_64-pc-windows-msvc.exe" -OutFile "C:\<Somewhere in PATH>"
```

### From source

Install Cargo along Rust using rustup: https://www.rust-lang.org/learn/get-started
```bash
cargo install --locked --git https://github.com/raunow/jig.git --features <cloud|server>
```
</details>
