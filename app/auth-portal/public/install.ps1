#Requires -Version 5.1

<#
.SYNOPSIS
    Installs the SI CLI binary for Windows.

.DESCRIPTION
    Downloads and installs the SI CLI binary release for Windows x86_64.
    Automatically detects system architecture and handles installation.

.PARAMETER Destination
    Destination directory for installation. Default: C:\Program Files\si (admin) or $env:LOCALAPPDATA\si (user)

.PARAMETER Version
    Release version to install. Default: stable
    Examples: stable, 20250218.210911.0-sha.bda1ce6ea

.PARAMETER AddToPath
    Add the installation directory to the system PATH. Default: $true

.PARAMETER Help
    Show this help message.

.EXAMPLE
    # Install to default location (user-specific)
    .\install.ps1

    # Install system-wide (requires admin)
    .\install.ps1

    # Install to custom location
    .\install.ps1 -Destination "C:\Tools\si"

    # Install specific version
    .\install.ps1 -Version "stable"

.NOTES
    Requires PowerShell 5.1 or later
#>

[CmdletBinding()]
param(
    [Parameter()]
    [string]$Destination,

    [Parameter()]
    [string]$Version = "stable",

    [Parameter()]
    [bool]$AddToPath = $true,

    [Parameter()]
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Script configuration
$BinName = "si"
$BinExe = "$BinName.exe"

function Write-Header {
    param([string]$Message)
    Write-Host "--- $Message" -ForegroundColor Cyan
}

function Write-Info {
    param([string]$Message)
    Write-Host "  - $Message" -ForegroundColor White
}

function Write-Success {
    param([string]$Message)
    Write-Host "  + $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "!!! $Message" -ForegroundColor Yellow
}

function Write-Err {
    param([string]$Message)
    Write-Host "xxx $Message" -ForegroundColor Red
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-DefaultDestination {
    $isAdmin = Test-Administrator

    if ($isAdmin) {
        return "C:\Program Files\$BinName"
    } else {
        return Join-Path $env:LOCALAPPDATA $BinName
    }
}

function Get-AssetUrl {
    param(
        [string]$Version,
        [string]$OsType,
        [string]$CpuType
    )

    $type = "binary"
    $platform = "$OsType-$CpuType"
    $extension = "zip"

    $url = "https://artifacts.systeminit.com/$BinName/$Version/$type"
    $url = "$url/$OsType/$CpuType/$BinName-$Version-$type-$platform.$extension"

    return $url
}

function Download-File {
    param(
        [string]$Url,
        [string]$Destination
    )

    Write-Info "Downloading $Url"

    try {
        # Use WebClient for download
        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($Url, $Destination)
        $webClient.Dispose()
    } catch {
        throw "Failed to download file: $_"
    }
}

function Extract-Archive {
    param(
        [string]$ArchivePath,
        [string]$DestinationPath
    )

    Write-Info "Extracting archive..."

    try {
        # Use .NET for extraction (works on all PowerShell versions)
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [System.IO.Compression.ZipFile]::ExtractToDirectory($ArchivePath, $DestinationPath)

        # Verify binary was extracted
        $extractedBinary = Join-Path $DestinationPath $BinExe
        if (-not (Test-Path $extractedBinary)) {
            throw "Failed to extract binary '$BinExe' from archive"
        }
    } catch {
        throw "Failed to extract archive: $_"
    }
}

function Install-Binary {
    param(
        [string]$SourcePath,
        [string]$DestinationDir
    )

    Write-Info "Installing '$BinExe' to '$DestinationDir'"

    try {
        # Create destination directory if it doesn't exist
        if (-not (Test-Path $DestinationDir)) {
            New-Item -ItemType Directory -Path $DestinationDir -Force | Out-Null
        }

        $destFile = Join-Path $DestinationDir $BinExe

        # Remove existing file if present
        if (Test-Path $destFile) {
            Remove-Item $destFile -Force
        }

        # Copy the binary
        Copy-Item -Path $SourcePath -Destination $destFile -Force

        Write-Success "Installed to $destFile"
    } catch {
        throw "Failed to install binary: $_"
    }
}

function Add-ToPath {
    param(
        [string]$Directory
    )

    $isAdmin = Test-Administrator
    $target = if ($isAdmin) { "Machine" } else { "User" }

    # Get current PATH
    $currentPath = [Environment]::GetEnvironmentVariable("Path", $target)

    # Check if directory is already in PATH
    $pathEntries = $currentPath -split ";" | ForEach-Object { $_.Trim() }
    if ($pathEntries -contains $Directory) {
        Write-Info "Directory already in PATH"
        return
    }

    Write-Info "Adding directory to $target PATH"

    try {
        $newPath = "$currentPath;$Directory"
        [Environment]::SetEnvironmentVariable("Path", $newPath, $target)

        # Update current session PATH
        $env:Path = "$env:Path;$Directory"

        Write-Success "Added to $target PATH"
        Write-Warn "You may need to restart your terminal for PATH changes to take effect"
    } catch {
        Write-Warn "Failed to add to PATH: $_"
        Write-Info "You can manually add '$Directory' to your PATH"
    }
}

function Show-Help {
    Get-Help $PSCommandPath -Detailed
}

function Main {
    if ($Help) {
        Show-Help
        exit 0
    }

    Write-Header "Installing '$BinName' for Windows"

    # Detect platform
    $osType = "windows"
    $cpuType = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }

    if ($cpuType -ne "x86_64") {
        Write-Err "Unsupported architecture: $cpuType. Only x86_64 is supported."
        exit 1
    }

    $platform = "$osType-$cpuType"
    Write-Info "Detected platform: $platform"

    # Determine destination
    if (-not $Destination) {
        $Destination = Get-DefaultDestination
    }

    $isAdmin = Test-Administrator
    if ($isAdmin) {
        Write-Info "Running with administrator privileges"
    } else {
        Write-Info "Running without administrator privileges (user install)"
    }

    Write-Info "Installation directory: $Destination"
    Write-Info "Version: $Version"

    # Create temp directory
    $tempDir = Join-Path $env:TEMP "si-install-$(Get-Random)"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    try {
        # Get download URL
        Write-Header "Downloading '$BinName' release '$Version'"
        $assetUrl = Get-AssetUrl -Version $Version -OsType $osType -CpuType $cpuType
        Write-Info "URL: $assetUrl"

        $zipFile = Join-Path $tempDir "si.zip"
        Download-File -Url $assetUrl -Destination $zipFile
        Write-Success "Downloaded successfully"

        # Extract archive
        Write-Header "Extracting archive"
        $extractDir = Join-Path $tempDir "extract"
        Extract-Archive -ArchivePath $zipFile -DestinationPath $extractDir
        Write-Success "Extracted successfully"

        # Install binary
        Write-Header "Installing binary"
        $binaryPath = Join-Path $extractDir $BinExe
        Install-Binary -SourcePath $binaryPath -DestinationDir $Destination

        # Add to PATH
        if ($AddToPath) {
            Write-Header "Configuring PATH"
            Add-ToPath -Directory $Destination
        }

        # Verify installation
        Write-Header "Verifying installation"
        $installedBinary = Join-Path $Destination $BinExe
        if (Test-Path $installedBinary) {
            Write-Success "Installation complete: $installedBinary"

            # Try to run --version
            try {
                $versionOutput = & $installedBinary --version 2>&1
                Write-Info "Version: $versionOutput"
            } catch {
                Write-Info "Binary installed successfully"
            }
        } else {
            throw "Installation verification failed"
        }

    } catch {
        Write-Err "Installation failed: $_"
        Write-Info ""
        Write-Warn "If you need help, please join us on our Discord!"
        Write-Warn "    https://discord.gg/system-init"
        exit 1
    } finally {
        # Cleanup
        if (Test-Path $tempDir) {
            Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    Write-Header "Installation of '$BinName' release '$Version' complete"
    Write-Info ""
    Write-Success "Run 'si --help' to get started"
    if (-not $AddToPath) {
        $binaryLocation = Join-Path $Destination $BinExe
        Write-Info "Binary location: $binaryLocation"
    }
}

# Run main function
Main
