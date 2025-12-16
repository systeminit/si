---
outline: [2, 3, 4]
---

# Install the SI CLI

## Quick Install (Recommended)

The easiest way to install the SI CLI is using our installation script

Our installationscripts support:
- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **windows**: x86_64

The installation script will:
- Automatically detect your system architecture (x86_64)
- Download the latest stable release
- Extract and install the binary
- Add to your PATH automatically

<DocTabs tabs="Linux & macOS, Windows">

<TabPanel value="Linux & macOS">

### Linux & macOS

**Basic Installation:**

```bash
# Install for current user (recommended)
curl -fsSL https://auth.systeminit.com/install.sh | sh
```

This installs to your user directory (`$HOME/.local/bin` or `$HOME/bin`). No sudo required!

**System-wide Installation:**

```bash
# Install for all users (requires sudo)
curl -fsSL https://auth.systeminit.com/install.sh | sudo sh
```

This installs to `/usr/local/bin`, making it available to all users on the system.

**Installation Options:**

```bash
# Install specific version
curl -fsSL https://auth.systeminit.com/install.sh | sudo sh -s -- -V stable

# Install to custom location
curl -fsSL https://auth.systeminit.com/install.sh | sudo sh -s -- -d ~/.local/bin

# Install specific platform
curl -fsSL https://auth.systeminit.com/install.sh | sudo sh -s -- -p darwin-aarch64

# See all options
curl -fsSL https://auth.systeminit.com/install.sh | sudo sh -s -- --help
```

</TabPanel>
<TabPanel value="Windows">

### Windows 

**Basic Install:**

```powershell
# Run the installation script from PowerShell
irm https://auth.systeminit.com/install.ps1 | iex
```

This installs to `$env:LOCALAPPDATA\si`

**System-wide Installation:**
```powershell
# Right-click PowerShell and select "Run as Administrator", then:
irm https://auth.systeminit.com/install.ps1 | iex
```

**Installation Options:**

```powershell
# Download and run with options
Invoke-WebRequest https://auth.systeminit.com/install.ps1 -OutFile install.ps1
.\install.ps1 -Help
.\install.ps1 -Destination "C:\Tools\si"
.\install.ps1 -Version "stable"
```

</TabPanel>

</DocTabs>



## Manual Install

If you prefer to download and install manually:

<DocTabs tabs="Linux (aarch64), Linux (x86_64), macOS (Apple Silicon), macOS (Intel), Windows (x86_64)">

<TabPanel value="Linux (aarch64)">

```bash
curl -LO https://artifacts.systeminit.com/si/stable/binary/linux/aarch64/si-stable-binary-linux-aarch64.tar.gz
tar -xzf si-stable-binary-linux-aarch64.tar.gz
sudo mv si /usr/local/bin/
# Or move to user bin: mv si ~/.local/bin/
```

</TabPanel>

<TabPanel value="Linux (x86_64)">

```bash
curl -LO https://artifacts.systeminit.com/si/stable/binary/linux/x86_64/si-stable-binary-linux-x86_64.tar.gz
tar -xzf si-stable-binary-linux-x86_64.tar.gz
sudo mv si /usr/local/bin/
# Or move to user bin: mv si ~/.local/bin/
```

</TabPanel>
<TabPanel value="macOS (Apple Silicon)">

```bash
curl -LO https://artifacts.systeminit.com/si/stable/binary/darwin/aarch64/si-stable-binary-darwin-aarch64.tar.gz
tar -xzf si-stable-binary-darwin-aarch64.tar.gz
sudo mv si /usr/local/bin/
# Or move to user bin: mv si ~/.local/bin/

# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/si
```

</TabPanel>
<TabPanel value="macOS (Intel)">

```bash
curl -LO https://artifacts.systeminit.com/si/stable/binary/darwin/x86_64/si-stable-binary-darwin-x86_64.tar.gz
tar -xzf si-stable-binary-darwin-x86_64.tar.gz
sudo mv si /usr/local/bin/
# Or move to user bin: mv si ~/.local/bin/

# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/si
```

</TabPanel>
<TabPanel value="Windows (x86_64)">

**Using PowerShell:**

```powershell
# Download the binary
Invoke-WebRequest -Uri https://artifacts.systeminit.com/si/stable/binary/windows/x86_64/si-stable-binary-windows-x86_64.zip -OutFile si.zip

# Extract the binary
Expand-Archive -Path si.zip -DestinationPath .

# Move to a directory in your PATH (example: C:\Program Files\si)
New-Item -ItemType Directory -Force -Path "C:\Program Files\si"
Move-Item -Path .\si.exe -Destination "C:\Program Files\si\si.exe"

# Add to PATH (requires admin privileges)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Program Files\si", [EnvironmentVariableTarget]::Machine)
```

**Using Command Prompt (cmd):**

```cmd
REM Download the binary (requires curl, available in Windows 10+)
curl -LO https://artifacts.systeminit.com/si/stable/binary/windows/x86_64/si-stable-binary-windows-x86_64.zip

REM Extract using tar (available in Windows 10+)
tar -xf si-stable-binary-windows-x86_64.zip

REM Move to a directory in your PATH
move si.exe "C:\Program Files\si\si.exe"

REM Add to PATH (requires admin privileges)
setx /M PATH "%PATH%;C:\Program Files\si"
```

</TabPanel>
</DocTabs>