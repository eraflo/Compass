# Compass Quality Check Script
# This script runs fmt, clippy, and tests.

Write-Host "--- Running Cargo Fmt ---" -ForegroundColor Cyan
cargo fmt --all
if ($LASTEXITCODE -ne 0) { Write-Host "Fmt failed!" -ForegroundColor Red; exit $LASTEXITCODE }

Write-Host "`n--- Running Cargo Clippy ---" -ForegroundColor Cyan
cargo clippy -- -D warnings
if ($LASTEXITCODE -ne 0) { Write-Host "Clippy failed!" -ForegroundColor Red; exit $LASTEXITCODE }

Write-Host "`n--- Running Cargo Test ---" -ForegroundColor Cyan
cargo test
if ($LASTEXITCODE -ne 0) { Write-Host "Tests failed!" -ForegroundColor Red; exit $LASTEXITCODE }

Write-Host "`n[SUCCESS] All checks passed!" -ForegroundColor Green
