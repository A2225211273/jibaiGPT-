$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$draftsDirectory = Join-Path $repoRoot "design\drafts"
$tempDirectory = Join-Path $env:TEMP "hexu-ui-captures"
$edgeProfile = Join-Path $env:TEMP "hexu-edge-profile"
$edgeCandidates = @(
  "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
  "C:\Program Files\Microsoft\Edge\Application\msedge.exe"
)
$edge = $edgeCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1

if (-not $edge) {
  throw "Microsoft Edge was not found. UI captures cannot be generated."
}

try {
  $null = Invoke-WebRequest -Uri "http://127.0.0.1:4173" -UseBasicParsing -TimeoutSec 3
} catch {
  throw "Cannot reach http://127.0.0.1:4173. Run pnpm dev first."
}

New-Item -ItemType Directory -Force -Path $draftsDirectory, $tempDirectory, $edgeProfile | Out-Null

$captures = @(
  @{ File = "hexu-ui-light-v2.png"; Width = 1536; Height = 1024; Query = "theme=light&view=board&capture=1" },
  @{ File = "hexu-ui-dark-v2.png"; Width = 1536; Height = 1024; Query = "theme=dark&view=board&capture=1" },
  @{ File = "hexu-ui-desktop-light-v2.png"; Width = 960; Height = 760; Query = "theme=light&view=desktop&capture=1" },
  @{ File = "hexu-ui-attached-light-v2.png"; Width = 1240; Height = 760; Query = "theme=light&view=attached&capture=1" },
  @{ File = "hexu-ui-conversation-light-v2.png"; Width = 1240; Height = 620; Query = "theme=light&view=conversation&capture=1" }
)

foreach ($capture in $captures) {
  $temporaryShot = Join-Path $tempDirectory $capture.File
  $destination = Join-Path $draftsDirectory $capture.File
  $url = "http://127.0.0.1:4173/?$($capture.Query)"
  $arguments = @(
    "--headless=new",
    "--disable-gpu",
    "--hide-scrollbars",
    "--force-device-scale-factor=1",
    "--window-size=$($capture.Width),$($capture.Height)",
    "--user-data-dir=$edgeProfile",
    "--no-first-run",
    "--screenshot=$temporaryShot",
    $url
  )

  Remove-Item -LiteralPath $temporaryShot -Force -ErrorAction SilentlyContinue
  $process = Start-Process -FilePath $edge -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
  if ($process.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $temporaryShot)) {
    throw "Capture failed: $($capture.File)"
  }

  Move-Item -LiteralPath $temporaryShot -Destination $destination -Force
  Write-Output "Generated $destination"
}
