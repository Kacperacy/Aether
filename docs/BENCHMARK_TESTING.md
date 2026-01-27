# Testowanie benchmarków algorytmów

## Komendy

### benchcompare [time_ms]
Porównuje wszystkie algorytmy na bieżącej pozycji.
- `time_ms` - limit czasu w ms (domyślnie: 1000)

### benchexport <input.epd> <output.csv> [time_ms]
Eksportuje wyniki do CSV.
- `time_ms` - limit czasu w ms (domyślnie: 1000)

## Przykłady

```bash
# Podstawowy test
echo -e "position startpos\nbenchcompare 1000\nquit" | ./target/release/aether

# Krótki test
echo -e "position startpos\nbenchcompare 100\nquit" | ./target/release/aether

# Eksport CSV
echo -e "benchexport positions/opening/noob_3moves.epd results.csv 500\nquit" | ./target/release/aether
```

## Interpretacja wyników

| Kolumna | Opis |
|---------|------|
| Depth | Osiągnięta głębokość |
| Nodes | Przeszukane węzły |
| Time (ms) | Rzeczywisty czas |
| NPS | Węzły/sekundę |
| TTFM | Czas do pierwszego ruchu |
| BestMv | Najlepszy ruch |
| Stab | Stabilność |

## Scenariusze testowe

### Test 1: Kompilacja
```bash
cargo build --release
```

### Test 2: Podstawowy (1s)
```bash
echo -e "position startpos\nbenchcompare 1000\nquit" | ./target/release/aether
```

### Test 3: Krótki (100ms)
```bash
echo -e "position startpos\nbenchcompare 100\nquit" | ./target/release/aether
```

### Test 4: Eksport CSV
```bash
head -3 positions/opening/noob_3moves.epd > /tmp/test.epd
echo -e "benchexport /tmp/test.epd /tmp/result.csv 500\nquit" | ./target/release/aether
cat /tmp/result.csv
```

### Test 5: Skalowanie
```bash
echo -e "position startpos\nbenchcompare 100\nquit" | ./target/release/aether
echo -e "position startpos\nbenchcompare 500\nquit" | ./target/release/aether
echo -e "position startpos\nbenchcompare 1000\nquit" | ./target/release/aether
```

## Typowe wartości (1 sekunda)

| Algorytm | NPS | Głębokość |
|----------|-----|-----------|
| Pure Alpha-Beta | 8-12M | 6-8 |
| Full Alpha-Beta | 3-5M | 12-14 |
| MTD(f) | 3-5M | 6-8 |
| NegaScout | 7-10M | 6-8 |
| MCTS | 2-3M | 17-20 |
| Classic MCTS | 40-50K | 11-13 |

## Skrypt automatyczny

```bash
./scripts/run_comparison.sh 1000  # 1 sekunda na algorytm
```
