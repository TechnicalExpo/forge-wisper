$ErrorActionPreference = "Stop"

$repoRoot = (Get-Location).Path
$repoTarget = Join-Path $repoRoot "target"
$shortTarget = "C:\t"

Write-Host "This removes generated Rust build artifacts only." -ForegroundColor Yellow
Write-Host "Repository target: $repoTarget"
Write-Host "Short Vulkan target: $shortTarget"
$confirmation = Read-Host "Type CLEAN to continue"
if ($confirmation -cne "CLEAN") {
  Write-Host "Cancelled."
  exit 0
}

if (Test-Path -LiteralPath $repoTarget) { Remove-Item -LiteralPath $repoTarget -Recurse -Force }
if (Test-Path -LiteralPath $shortTarget) { Remove-Item -LiteralPath $shortTarget -Recurse -Force }
Write-Host "Generated Rust build artifacts removed." -ForegroundColor Green
