# Uruchom pojedynczą grę Aether vs Stockfish (do debugowania)

param(
    [int]$Level = 5,
    [string]$TimeControl = "5+3"
)

Write-Host "=== Pojedyncza gra: Aether vs Stockfish poziom $Level ===" -ForegroundColor Cyan
Write-Host "Time control: $TimeControl"
Write-Host ""

$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$Output = "single_game_${Timestamp}.pgn"

$AetherPath = "$PSScriptRoot\target\release\aether-uci.exe"

& cutechess-cli `
    -engine cmd="$AetherPath" name="Aether" option."Move Overhead"=100 `
    -engine cmd="stockfish.exe" name="Stockfish-L$Level" option."Skill Level"=$Level `
    -each tc="$TimeControl" `
    -rounds 1 `
    -pgnout "$Output" `
    -debug

Write-Host ""
Write-Host "Gra zapisana w: $Output" -ForegroundColor Green
Write-Host ""
Write-Host "Wynik:"
Get-Content $Output | Select-String "^\[Result"
