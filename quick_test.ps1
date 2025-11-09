# Szybki test - 10 gier vs Stockfish poziom 5

Write-Host "=== Szybki test Aether vs Stockfish (10 gier) ===" -ForegroundColor Cyan
Write-Host ""

.\run_tournament.ps1 -Level 5 -Games 10 -TimeControl "10+0.1" -Concurrency 2 -Output "quick_test.pgn"
