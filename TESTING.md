# Testing Aether Chess Engine

Przewodnik po testowaniu silnika Aether przeciwko Stockfish przy użyciu cutechess-cli.

## Wymagania

### Instalacja narzędzi

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y cutechess stockfish
```

**Fedora/RHEL:**
```bash
sudo dnf install -y cutechess stockfish
```

**Arch Linux:**
```bash
sudo pacman -S cutechess stockfish
```

**Ręczna instalacja:**
- Cutechess: https://github.com/cutechess/cutechess/releases
- Stockfish: https://stockfishchess.org/download/

### Budowanie Aether

```bash
cargo build --release
```

## Dostępne skrypty testowe

### 1. `run_tournament.sh` - Główny skrypt turnieju

Uniwersalny skrypt do uruchamiania turniejów z pełną konfiguracją.

**Podstawowe użycie:**
```bash
./run_tournament.sh
```

**Opcje:**
- `--level N` - Poziom Stockfish (0-20, domyślnie 5)
- `--games N` - Liczba gier (domyślnie 100)
- `--tc TIME` - Time control (domyślnie "40/60+0.6")
- `--concurrency N` - Liczba równoległych gier (domyślnie 1)
- `--output FILE` - Plik wyjściowy PGN

**Przykłady:**
```bash
# 50 gier vs Stockfish poziom 8 w time control 5+3
./run_tournament.sh --level 8 --games 50 --tc "5+3"

# Szybkie 20 gier z 2 równoległymi grami
./run_tournament.sh --games 20 --tc "10+0.1" --concurrency 2

# Długie testy z zapisem do pliku
./run_tournament.sh --games 200 --tc "40/120+1" --output long_test.pgn
```

### 2. `quick_test.sh` - Szybki test (10 gier)

Test sprawdzający czy silnik działa poprawnie.

```bash
./quick_test.sh
```

**Parametry:**
- 10 gier (5 rund)
- Time control: 10+0.1 (10s + 0.1s increment)
- Stockfish poziom 5
- 2 równoległe gry

**Czas trwania:** ~3-5 minut

### 3. `lichess_simulation.sh` - Symulacja Lichess 5+3

Dokładna symulacja warunków z Lichess.

```bash
./lichess_simulation.sh
```

**Parametry:**
- 50 gier (25 rund)
- Time control: 5+3 (jak na Lichess)
- Stockfish poziom 5 (~1500-1600 ELO)
- 1 gra na raz (jak na Lichess)

**Czas trwania:** ~2-3 godziny

### 4. `full_test.sh` - Pełny test (100 gier × 3 poziomy)

Kompleksowy test przeciwko różnym poziomom Stockfish.

```bash
./full_test.sh
```

**Parametry:**
- 100 gier vs każdy poziom (3, 5, 7)
- Time control: 40/60+0.6
- 2 równoległe gry
- Tworzy 3 pliki PGN

**Czas trwania:** ~4-6 godzin

## Analiza wyników

### `analyze_results.sh` - Analiza plików PGN

```bash
./analyze_results.sh results.pgn
```

**Wyświetla:**
- Liczba wygranych/remisów/porażek
- Wyniki jako białe/czarne
- Procent punktów
- Szacunkowe ELO
- Średnia długość gry
- Przyczyny zakończenia gier

**Analiza wielu plików:**
```bash
./analyze_results.sh *.pgn
```

## Time Control - formaty

Cutechess-cli obsługuje różne formaty time control:

### Podstawowe formaty:

```bash
"5+3"          # 5 sekund + 3 sekundy increment per ruch
"10+0.1"       # 10 sekund + 0.1s increment
"60+1"         # 1 minuta + 1s increment
"180+2"        # 3 minuty + 2s increment
```

### Formaty z kontrolą ruchów:

```bash
"40/60"        # 60 sekund na pierwsze 40 ruchów
"40/60+0.6"    # 60s na 40 ruchów + 0.6s increment
"40/120+1"     # 2 minuty na 40 ruchów + 1s increment
```

### Zalecane time controls:

| Nazwa | Format | Opis | Czas gry |
|-------|--------|------|----------|
| **Bullet** | `1+0` | Szachy błyskawiczne | ~1 min |
| **Bullet+** | `2+1` | Szybkie z increment | ~2 min |
| **Blitz** | `5+3` | Standard Lichess | ~8 min |
| **Rapid** | `10+5` | Szybkie partia | ~15 min |
| **Classical** | `40/120+30` | Klasyczne szachy | ~1h |

## Poziomy Stockfish

Stockfish Skill Level i przybliżone ELO:

| Poziom | Szacunkowe ELO | Opis |
|--------|----------------|------|
| 0-2 | <1000 | Początkujący |
| 3-4 | 1200-1400 | Średniozaawansowany |
| 5-6 | 1500-1650 | Zaawansowany |
| 7-9 | 1700-1900 | Ekspert |
| 10-12 | 2000-2200 | Mistrz |
| 13-15 | 2300-2500 | Arcymistrz |
| 16-20 | 2600+ | Super GM |

**Cel dla Aether:** Wygrywać z poziomem 5-6 (1500-1650 ELO)

## Interpretacja wyników

### Procent punktów:

```
>75%  - Dominacja (silnik znacznie silniejszy)
60-75% - Wyraźna przewaga
55-60% - Przewaga
50-55% - Równy poziom
45-50% - Lekko słabszy
<45%  - Wyraźnie słabszy
```

### Przykładowe dobre wyniki:

```
Aether vs Stockfish Level 5:
  Wygrane: 45
  Remisy: 20
  Przegrane: 35
  Wynik: 55/100 (55%) ✅ DOBRY WYNIK
```

### Czerwone flagi:

```
❌ >10% przegranych na czas - problem z zarządzaniem czasem
❌ <40% przeciwko level 5 - za słaby
❌ Średnia długość gry <15 ruchów - blunders
```

## Przykładowy workflow testowania

### 1. Szybki test sanity check
```bash
./quick_test.sh
./analyze_results.sh quick_test.pgn
```

### 2. Test symulacyjny Lichess
```bash
./lichess_simulation.sh
./analyze_results.sh lichess_5plus3.pgn
```

### 3. Pełny benchmark
```bash
./full_test.sh
./analyze_results.sh full_test_level*.pgn
```

### 4. Custom test
```bash
# Test przeciwko poziomowi 6 w długim time control
./run_tournament.sh --level 6 --games 50 --tc "15+10" --output custom_test.pgn
./analyze_results.sh custom_test.pgn
```

## Rozwiązywanie problemów

### cutechess-cli nie uruchamia się
```bash
which cutechess-cli
# Jeśli nie znaleziono, zainstaluj ponownie
```

### Stockfish nie jest wykrywany
```bash
which stockfish
# Jeśli w innej lokalizacji, edytuj STOCKFISH_PATH w run_tournament.sh
```

### Silnik Aether nie działa
```bash
# Zbuduj ponownie
cargo clean
cargo build --release

# Sprawdź czy działa
echo -e "uci\nquit" | ./target/release/aether-uci
```

### Zbyt wolne testy
```bash
# Zwiększ concurrency (więcej równoległych gier)
./run_tournament.sh --concurrency 4 --games 100

# Użyj krótszego time control
./run_tournament.sh --tc "3+2" --games 50
```

## Zaawansowane opcje cutechess-cli

### Opening book
```bash
cutechess-cli \
    -engine cmd=aether-uci \
    -engine cmd=stockfish \
    -openings file=openings.pgn format=pgn order=random \
    -each tc=5+3 \
    -rounds 50
```

### Zapis szczegółowych logów
```bash
cutechess-cli \
    -engine cmd=aether-uci \
    -engine cmd=stockfish \
    -each tc=5+3 \
    -rounds 50 \
    -debug \
    -pgnout games.pgn \
    -epdout positions.epd
```

### Testowanie z różnymi ustawieniami
```bash
cutechess-cli \
    -engine cmd=aether-uci option."Move Overhead"=200 option."Hash"=128 \
    -engine cmd=stockfish option."Skill Level"=5 \
    -each tc=5+3 \
    -rounds 50
```

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
- 🎯 Wygrywanie turniejów na Lichess (1500-1600 pool)

## Wsparcie

W razie problemów:
- Sprawdź logi w plikach PGN
- Uruchom `./analyze_results.sh` dla diagnozy
- Testuj najpierw z `quick_test.sh`
