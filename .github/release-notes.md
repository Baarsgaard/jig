## Features
-
-

## Changes
-
-

## Fixes
-
-

All changes: https://github.com/baarsgaard/jig/compare/${OLD_VERSION}...v${CRATE_VERSION}

### Upgrading:

For the newest release:
```bash
jig upgrade
```


<details>
<summary><h2>Installation</h2></summary>

Pick between `cloud` and `data-center` instances (APIs differ)

### Linux

```bash
mkdir -p ~/.local/bin || true
wget -O ~/.local/bin/jig "https://github.com/baarsgaard/jig/releases/latest/download/jig-$INSTANCE-$TRIPLE"
chmod +x ~/.local/bin/jig"
```

### Windows
```posh
Invoke-WebRequest -Uri "https://github.com/baarsgaard/jig/releases/download/latest/jig-$INSTANCE-x86_64-pc-windows-msvc.exe" -OutFile "C:\<Somewhere in PATH>"
```

### From source

Install Cargo along Rust using rustup: https://www.rust-lang.org/learn/get-started
```bash
cargo install --locked --git https://github.com/raunow/jig.git --features <cloud|data-center>
```
</details>
