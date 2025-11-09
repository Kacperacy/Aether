# Symulacja warunków Lichess 5+3

Write-Host "=== Symulacja Lichess 5+3 ===" -ForegroundColor Cyan
Write-Host "Stockfish poziom 5 (odpowiednik ~1500-1600 ELO)" -ForegroundColor Yellow
Write-Host ""

.\run_tournament.ps1 -Level 5 -Games 50 -TimeControl "5+3" -Concurrency 1 -Output "lichess_5plus3.pgn"
