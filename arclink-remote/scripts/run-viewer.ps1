# PowerShell Script to run ArcLink Viewer
Write-Host "Building ArcLink Viewer..." -ForegroundColor Cyan
cargo build --bin arclink-viewer

if ($LASTEXITCODE -eq 0) {
    Write-Host "Launching ArcLink Viewer..." -ForegroundColor Green
    cargo run --bin arclink-viewer
} else {
    Write-Host "Failed to compile Viewer applet." -ForegroundColor Red
}
