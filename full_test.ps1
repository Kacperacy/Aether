# Pełny test - 100 gier vs różne poziomy Stockfish

Write-Host "=== Pełny test Aether vs Stockfish (100 gier per poziom) ===" -ForegroundColor Cyan
Write-Host ""

foreach ($Level in 3, 5, 7) {
    Write-Host ""
    Write-Host "=====================================" -ForegroundColor Green
    Write-Host "  Testowanie przeciwko poziomowi $Level" -ForegroundColor Green
    Write-Host "=====================================" -ForegroundColor Green
    
    .\run_tournament.ps1 -Level $Level -Games 100 -TimeControl "40/60+0.6" -Concurrency 2 -Output "full_test_level$Level.pgn"
    
    Write-Host ""
    Write-Host "Zakończono testy dla poziomu $Level" -ForegroundColor Yellow
    Write-Host ""
}

Write-Host ""
Write-Host "=====================================" -ForegroundColor Green
Write-Host "  Wszystkie testy zakończone!" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host "Wyniki w plikach: full_test_level*.pgn"
