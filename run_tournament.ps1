# Skrypt do testowania Aether vs Stockfish na Windows przy użyciu cutechess-cli
#
# Użycie:
#   .\run_tournament.ps1 -Level 5 -Games 100 -TimeControl "5+3"
#
# Parametry:
#   -Level         Poziom Stockfish (0-20, domyślnie 5)
#   -Games         Liczba gier (domyślnie 100)
#   -TimeControl   Time control (domyślnie "40/60+0.6")
#   -Concurrency   Liczba równoległych gier (domyślnie 1)
#   -Output        Plik wyjściowy PGN

param(
    [int]$Level = 5,
    [int]$Games = 100,
    [string]$TimeControl = "40/60+0.6",
    [int]$Concurrency = 1,
    [string]$Output = ""
)

# Kolory dla czytelności
function Write-Color {
    param([string]$Text, [string]$Color = "White")
    Write-Host $Text -ForegroundColor $Color
}

# Ścieżki do silników
$AetherPath = "$PSScriptRoot\target\release\aether.exe"
$StockfishPath = "stockfish.exe"  # Zakładamy że jest w PATH

# Sprawdź czy cutechess-cli jest zainstalowany
if (-not (Get-Command cutechess-cli -ErrorAction SilentlyContinue)) {
    Write-Color "BŁĄD: cutechess-cli nie jest zainstalowany lub nie jest w PATH" "Red"
    Write-Color "Pobierz z: https://github.com/cutechess/cutechess/releases" "Yellow"
    Write-Color "I dodaj do zmiennej środowiskowej PATH" "Yellow"
    exit 1
}

# Sprawdź czy Stockfish jest zainstalowany
if (-not (Get-Command $StockfishPath -ErrorAction SilentlyContinue)) {
    Write-Color "BŁĄD: Stockfish nie jest zainstalowany lub nie jest w PATH" "Red"
    Write-Color "Pobierz z: https://stockfishchess.org/download/" "Yellow"
    exit 1
}

# Sprawdź czy Aether został zbudowany
if (-not (Test-Path $AetherPath)) {
    Write-Color "BŁĄD: Nie znaleziono Aether UCI w: $AetherPath" "Red"
    Write-Color "Zbuduj silnik: cargo build --release" "Yellow"
    exit 1
}

# Utwórz nazwę pliku wyjściowego jeśli nie podano
if ($Output -eq "") {
    $Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $Output = "results_sf${Level}_${Timestamp}.pgn"
}

# Wyświetl konfigurację
Write-Color "=========================================" "Cyan"
Write-Color "  Turniej: Aether vs Stockfish" "Cyan"
Write-Color "=========================================" "Cyan"
Write-Host "Aether:           $AetherPath"
Write-Host "Stockfish:        $StockfishPath (poziom $Level)"
Write-Host "Liczba gier:      $Games"
Write-Host "Time control:     $TimeControl"
Write-Host "Równoległość:     $Concurrency"
Write-Host "Plik wynikowy:    $Output"
Write-Color "=========================================" "Cyan"
Write-Host ""
Write-Color "Rozpoczynanie turnieju..." "Green"
Write-Host ""

# Oblicz liczbę rund
$Rounds = [math]::Floor($Games / 2)

# Uruchom turniej
& cutechess-cli `
    -engine cmd="$AetherPath" name="Aether" option."Move Overhead"=100 `
    -engine cmd="$StockfishPath" name="Stockfish-L$Level" option."Skill Level"=$Level `
    -each tc="$TimeControl" `
    -rounds $Rounds `
    -games 2 `
    -repeat `
    -concurrency $Concurrency `
    -pgnout "$Output" `
    -ratinginterval 10 `
    -recover

Write-Host ""
Write-Color "=========================================" "Cyan"
Write-Color "Turniej zakończony!" "Green"
Write-Color "=========================================" "Cyan"
Write-Host "Wyniki zapisane w: $Output"
Write-Host ""

# Analiza wyników
if (Test-Path $Output) {
    Write-Color "Analiza wyników:" "Yellow"
    $Content = Get-Content $Output -Raw
    
    $AetherWins = ([regex]::Matches($Content, '1-0.*Aether')).Count
    $StockfishWins = ([regex]::Matches($Content, '0-1.*Aether')).Count
    $Draws = ([regex]::Matches($Content, '1/2-1/2')).Count
    
    Write-Host "Wygrane Aether:     $AetherWins"
    Write-Host "Wygrane Stockfish:  $StockfishWins"
    Write-Host "Remisy:             $Draws"
    Write-Host ""
    
    $Total = $AetherWins + $StockfishWins + $Draws
    if ($Total -gt 0) {
        $AetherScore = [math]::Round((($AetherWins + $Draws * 0.5) / $Total * 100), 1)
        Write-Host "Wynik Aether: $AetherScore%"
    }
}
Write-Host ""
Write-Color "Aby zobaczyć szczegóły, otwórz plik PGN w programie do analizy szachowej." "Cyan"
