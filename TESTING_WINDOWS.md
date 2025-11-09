# Testing Aether Chess Engine - Windows

Przewodnik po testowaniu silnika Aether przeciwko Stockfish na Windows przy użyciu cutechess-cli.

## Wymagania

### 1. Instalacja Rust (jeśli nie masz)

Pobierz i zainstaluj z: https://rustup.rs/

```powershell
# Sprawdź instalację
cargo --version
```

### 2. Instalacja cutechess-cli

**Opcja A: Pobierz oficjalny release (ZALECANE)**

1. Pobierz najnowszą wersję: https://github.com/cutechess/cutechess/releases
2. Pobierz plik `cutechess-cli-*.zip` dla Windows
3. Rozpakuj do folderu (np. `C:\Tools\cutechess\`)
4. Dodaj ścieżkę do zmiennej środowiskowej PATH:
   - Otwórz: Panel sterowania → System → Zaawansowane ustawienia systemu
   - Kliknij "Zmienne środowiskowe"
   - W "Zmienne systemowe" znajdź "Path" i kliknij "Edytuj"
   - Dodaj: `C:\Tools\cutechess\`
   - Kliknij OK

**Opcja B: Zbuduj ze źródeł (dla zaawansowanych)**

```powershell
git clone https://github.com/cutechess/cutechess.git
cd cutechess
# Wymaga Qt i CMake
```

**Weryfikacja:**
```powershell
cutechess-cli --version
```

### 3. Instalacja Stockfish

**Pobierz z oficjalnej strony:**

1. Pobierz: https://stockfishchess.org/download/
2. Wybierz "Windows" i pobierz najnowszą wersję
3. Rozpakuj `stockfish.exe` do folderu (np. `C:\Tools\stockfish\`)
4. Dodaj ścieżkę do PATH (jak wyżej)

**Lub użyj Chocolatey:**
```powershell
choco install stockfish
```

**Weryfikacja:**
```powershell
stockfish.exe
# Powinien wyświetlić "Stockfish..."
# Wpisz "quit" aby wyjść
```

### 4. Budowanie Aether

```powershell
cd path\to\Aether
cargo build --release
```

Binarka będzie w: `target\release\aether.exe`

## Dostępne skrypty PowerShell

### 1. `run_tournament.ps1` - Główny skrypt turnieju

Uniwersalny skrypt do uruchamiania turniejów z pełną konfiguracją.

**Podstawowe użycie:**
```powershell
.\run_tournament.ps1
```

**Parametry:**
- `-Level N` - Poziom Stockfish (0-20, domyślnie 5)
- `-Games N` - Liczba gier (domyślnie 100)
- `-TimeControl TIME` - Time control (domyślnie "40/60+0.6")
- `-Concurrency N` - Liczba równoległych gier (domyślnie 1)
- `-Output FILE` - Plik wyjściowy PGN

**Przykłady:**
```powershell
# 50 gier vs Stockfish poziom 8 w time control 5+3
.\run_tournament.ps1 -Level 8 -Games 50 -TimeControl "5+3"

# Szybkie 20 gier z 2 równoległymi grami
.\run_tournament.ps1 -Games 20 -TimeControl "10+0.1" -Concurrency 2

# Długie testy z zapisem do pliku
.\run_tournament.ps1 -Games 200 -TimeControl "40/120+1" -Output "long_test.pgn"
```

### 2. `quick_test.ps1` - Szybki test (10 gier)

Test sprawdzający czy silnik działa poprawnie.

```powershell
.\quick_test.ps1
```

**Parametry:**
- 10 gier (5 rund)
- Time control: 10+0.1
- Stockfish poziom 5
- 2 równoległe gry

**Czas trwania:** ~3-5 minut

### 3. `lichess_simulation.ps1` - Symulacja Lichess 5+3

Dokładna symulacja warunków z Lichess.

```powershell
.\lichess_simulation.ps1
```

**Parametry:**
- 50 gier (25 rund)
- Time control: 5+3
- Stockfish poziom 5 (~1500-1600 ELO)
- 1 gra na raz

**Czas trwania:** ~2-3 godziny

### 4. `full_test.ps1` - Pełny test (100 gier × 3 poziomy)

Kompleksowy test przeciwko różnym poziomom Stockfish.

```powershell
.\full_test.ps1
```

**Parametry:**
- 100 gier vs każdy poziom (3, 5, 7)
- Time control: 40/60+0.6
- 2 równoległe gry
- Tworzy 3 pliki PGN

**Czas trwania:** ~4-6 godzin

### 5. `single_game.ps1` - Pojedyncza gra

Uruchom pojedynczą grę do debugowania.

```powershell
# Domyślnie: poziom 5, time control 5+3
.\single_game.ps1

# Z parametrami
.\single_game.ps1 -Level 6 -TimeControl "10+5"
```

## Analiza wyników

### `analyze_results.ps1` - Analiza plików PGN

```powershell
.\analyze_results.ps1 results.pgn
```

**Wyświetla:**
- Liczba wygranych/remisów/porażek
- Wyniki jako białe/czarne
- Procent punktów
- Szacunkowe ELO
- Średnia długość gry
- Przyczyny zakończenia gier

**Analiza wielu plików:**
```powershell
.\analyze_results.ps1 *.pgn
```

## Polityka wykonywania skryptów PowerShell

Windows może blokować wykonywanie skryptów PowerShell. Aby to naprawić:

### Opcja 1: Tymczasowo dla bieżącej sesji
```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

### Opcja 2: Trwale dla użytkownika (ZALECANE)
```powershell
# Uruchom PowerShell jako Administrator
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
```

### Opcja 3: Odblokuj konkretne skrypty
```powershell
Unblock-File -Path .\run_tournament.ps1
Unblock-File -Path .\quick_test.ps1
Unblock-File -Path .\analyze_results.ps1
# etc.
```

## Przykładowy workflow testowania na Windows

### 1. Przygotowanie środowiska

```powershell
# Otwórz PowerShell
cd C:\path\to\Aether

# Sprawdź czy wszystko działa
cargo build --release
cutechess-cli --version
stockfish.exe

# Odblokuj skrypty (jeśli potrzeba)
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

### 2. Szybki test sanity check

```powershell
.\quick_test.ps1
.\analyze_results.ps1 quick_test.pgn
```

### 3. Test symulacyjny Lichess

```powershell
.\lichess_simulation.ps1
.\analyze_results.ps1 lichess_5plus3.pgn
```

### 4. Pełny benchmark

```powershell
.\full_test.ps1
.\analyze_results.ps1 full_test_level*.pgn
```

### 5. Custom test

```powershell
.\run_tournament.ps1 -Level 6 -Games 50 -TimeControl "15+10" -Output "custom_test.pgn"
.\analyze_results.ps1 custom_test.pgn
```

## Time Control - formaty

Identyczne jak na Linux:

```powershell
"5+3"          # 5 sekund + 3 sekundy increment
"10+0.1"       # 10 sekund + 0.1s increment
"40/60+0.6"    # 60s na 40 ruchów + 0.6s increment
```

## Poziomy Stockfish i ELO

| Poziom | Szacunkowe ELO | Opis |
|--------|----------------|------|
| 0-2 | <1000 | Początkujący |
| 3-4 | 1200-1400 | Średniozaawansowany |
| 5-6 | 1500-1650 | Zaawansowany (CEL) |
| 7-9 | 1700-1900 | Ekspert |
| 10-12 | 2000-2200 | Mistrz |
| 13-20 | 2300+ | Arcymistrz |

## Rozwiązywanie problemów na Windows

### cutechess-cli nie jest rozpoznawany

```powershell
# Sprawdź czy jest w PATH
$env:Path -split ';' | Select-String cutechess

# Jeśli nie, dodaj tymczasowo:
$env:Path += ";C:\Tools\cutechess"

# Lub użyj pełnej ścieżki w skrypcie
```

### Stockfish nie jest rozpoznawany

```powershell
# Sprawdź
where.exe stockfish.exe

# Dodaj do PATH lub użyj pełnej ścieżki
```

### "Nie można uruchomić skryptu - polityka wykonywania"

```powershell
# Uruchom jako Administrator
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned

# Lub tymczasowo
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

### Aether nie buduje się

```powershell
# Sprawdź wersję Rust
rustc --version

# Zaktualizuj Rust
rustup update

# Clean rebuild
cargo clean
cargo build --release
```

### Zbyt wolne testy

```powershell
# Zwiększ concurrency
.\run_tournament.ps1 -Concurrency 4 -Games 100

# Użyj krótszego time control
.\run_tournament.ps1 -TimeControl "3+2" -Games 50
```

## Zaawansowane użycie

### Własna konfiguracja cutechess-cli

```powershell
$AetherPath = ".\target\release\aether.exe"
$StockfishPath = "C:\Tools\stockfish\stockfish.exe"

cutechess-cli `
    -engine cmd="$AetherPath" name="Aether" `
        option."Move Overhead"=200 `
        option."Hash"=128 `
    -engine cmd="$StockfishPath" name="Stockfish-L5" `
        option."Skill Level"=5 `
    -each tc="5+3" `
    -rounds 50 `
    -pgnout "my_games.pgn" `
    -debug
```

### Testowanie z opening book

```powershell
cutechess-cli `
    -engine cmd=".\target\release\aether.exe" `
    -engine cmd="stockfish.exe" option."Skill Level"=5 `
    -openings file="openings.pgn" format=pgn order=random `
    -each tc=5+3 `
    -rounds 50 `
    -pgnout "games.pgn"
```

### Batch testing różnych konfiguracji

```powershell
# Test różnych Move Overhead settings
foreach ($Overhead in 50, 100, 150, 200) {
    Write-Host "Testing Move Overhead: $Overhead ms" -ForegroundColor Cyan

    cutechess-cli `
        -engine cmd=".\target\release\aether.exe" option."Move Overhead"=$Overhead `
        -engine cmd="stockfish.exe" option."Skill Level"=5 `
        -each tc="5+3" `
        -rounds 20 `
        -pgnout "overhead_${Overhead}ms.pgn"
}
```

## Alternatywne narzędzia na Windows

### Arena Chess GUI

- Pobierz: http://www.playwitharena.de/
- GUI alternative to cutechess-cli
- Łatwiejsza konfiguracja wizualna

### Lucas Chess

- Pobierz: https://lucaschess.pythonanywhere.com/
- Pełny pakiet z wieloma silnikami
- Świetny do treningu

### ChessBase

- Komercyjne narzędzie profesjonalne
- Najlepsze do analizy i przygotowania

## Cele testowe dla Aether

### Minimum Viable Product (MVP):
- ✅ >40% vs Stockfish level 5
- ✅ <5% przegranych na czas
- ✅ Brak crash'ów podczas 100 gier

### Production Ready:
- ✅ >50% vs Stockfish level 5
- ✅ <2% przegranych na czas
- ✅ >45% vs Stockfish level 6

### Stretch Goals:
- 🎯 >55% vs Stockfish level 6
- 🎯 >45% vs Stockfish level 7
- 🎯 Wygrywanie turniejów na Lichess

## Przydatne komendy PowerShell

```powershell
# Wyświetl wszystkie pliki PGN
Get-ChildItem *.pgn

# Zlicz gry w pliku PGN
(Get-Content results.pgn | Select-String '^\[Result').Count

# Znajdź przegrane na czas
Get-Content results.pgn | Select-String -Pattern 'time' -Context 2

# Kopiuj wyniki do schowka
Get-Content results.pgn | Set-Clipboard

# Otwórz plik w domyślnym programie
Invoke-Item results.pgn
```

## Wsparcie

W razie problemów:
1. Sprawdź że wszystkie narzędzia są w PATH
2. Uruchom PowerShell jako Administrator
3. Ustaw ExecutionPolicy na RemoteSigned
4. Sprawdź logi w plikach PGN
5. Użyj `.\single_game.ps1` do debugowania

## Różnice Windows vs Linux

| Funkcja | Linux | Windows |
|---------|-------|---------|
| Skrypty | `.sh` (bash) | `.ps1` (PowerShell) |
| Binarka | `aether` | `aether.exe` |
| Separator ścieżek | `/` | `\` |
| Execution policy | brak | Wymaga ustawienia |
| PATH separator | `:` | `;` |

Wszystkie funkcje są identyczne, tylko składnia się różni!
