# PowerShell Script to run ArcLink Host
Write-Host "Building ArcLink Host..." -ForegroundColor Cyan
cargo build --bin arclink-host

if ($LASTEXITCODE -eq 0) {
    Write-Host "Launching ArcLink Host..." -ForegroundColor Green
    cargo run --bin arclink-host
} else {
    Write-Host "Failed to compile Host applet." -ForegroundColor Red
}
