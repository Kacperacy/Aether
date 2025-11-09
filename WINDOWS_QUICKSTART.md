# 🪟 Aether Chess Engine - Windows Quickstart

## 🚀 Szybki start (5 kroków)

### 1️⃣ Zainstaluj wymagane narzędzia

**A. Rust** (jeśli nie masz)
```
Pobierz: https://rustup.rs/
Uruchom instalator i postępuj zgodnie z instrukcjami
```

**B. cutechess-cli**
```
1. Pobierz: https://github.com/cutechess/cutechess/releases
2. Wybierz najnowszy release
3. Pobierz: cutechess-cli-*-win64.zip
4. Rozpakuj do: C:\Tools\cutechess\
5. Dodaj do PATH: C:\Tools\cutechess\
```

**C. Stockfish**
```
1. Pobierz: https://stockfishchess.org/download/
2. Wybierz: Windows (64-bit)
3. Rozpakuj stockfish.exe do: C:\Tools\stockfish\
4. Dodaj do PATH: C:\Tools\stockfish\
```

### 2️⃣ Dodaj narzędzia do PATH

**Krok po kroku:**
1. Wciśnij `Win + X` → wybierz "System"
2. Kliknij "Zaawansowane ustawienia systemu"
3. Kliknij "Zmienne środowiskowe"
4. W "Zmienne systemowe" znajdź "Path" i kliknij "Edytuj"
5. Kliknij "Nowy" i dodaj:
   - `C:\Tools\cutechess\`
   - `C:\Tools\stockfish\`
6. Kliknij OK we wszystkich oknach
7. **WAŻNE:** Zamknij i otwórz ponownie PowerShell/CMD

### 3️⃣ Zbuduj Aether

Otwórz PowerShell w folderze projektu:

```powershell
cd C:\path\to\Aether
cargo build --release
```

Poczekaj 2-5 minut. Binarka będzie w: `target\release\aether-uci.exe`

### 4️⃣ Odblokuj wykonywanie skryptów PowerShell

**Uruchom PowerShell jako Administrator:**

```powershell
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
```

Wpisz `Y` (Yes) i naciśnij Enter.

### 5️⃣ Uruchom szybki test!

**W zwykłym PowerShell (w folderze Aether):**

```powershell
.\quick_test.ps1
```

**Alternatywnie (jeśli PowerShell nie działa):**

Kliknij dwukrotnie na: `quick_test.bat`

---

## 📊 Dostępne skrypty

### PowerShell (zalecane)

```powershell
# Szybki test (10 gier, 3-5 min)
.\quick_test.ps1

# Symulacja Lichess 5+3 (50 gier, 2-3h)
.\lichess_simulation.ps1

# Pełny test vs poziomy 3,5,7 (300 gier, 4-6h)
.\full_test.ps1

# Pojedyncza gra (debugowanie)
.\single_game.ps1

# Custom turniej
.\run_tournament.ps1 -Level 6 -Games 50 -TimeControl "5+3"

# Analiza wyników
.\analyze_results.ps1 results.pgn
```

### Batch files (alternatywa)

```batch
REM Szybki test
quick_test.bat

REM Custom turniej
run_tournament.bat 5 100 "5+3"
```

---

## 🔧 Rozwiązywanie problemów

### ❌ "cutechess-cli nie jest rozpoznawany"

**Przyczyna:** cutechess-cli nie jest w PATH

**Rozwiązanie:**
```powershell
# Sprawdź PATH
$env:Path -split ';' | Select-String cutechess

# Jeśli nic nie zwraca, dodaj tymczasowo:
$env:Path += ";C:\Tools\cutechess"

# Lub dodaj na stałe przez Zmienne środowiskowe (krok 2)
```

### ❌ "Nie można uruchomić skryptu - polityka wykonywania"

**Przyczyna:** Windows blokuje skrypty PowerShell

**Rozwiązanie:**
```powershell
# Opcja 1: Uruchom PowerShell jako Administrator
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned

# Opcja 2: Tymczasowo dla bieżącej sesji
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass

# Opcja 3: Odblokuj konkretny skrypt
Unblock-File .\quick_test.ps1
```

### ❌ "Nie znaleziono Aether UCI"

**Przyczyna:** Silnik nie został zbudowany

**Rozwiązanie:**
```powershell
cargo build --release
```

### ❌ Stockfish nie działa

**Sprawdź instalację:**
```powershell
# Powinien wyświetlić "Stockfish..."
stockfish.exe

# Jeśli nie działa, użyj pełnej ścieżki w skryptach
```

---

## 📖 Pełna dokumentacja

Szczegółowe informacje w:
- **TESTING_WINDOWS.md** - Kompleksowy przewodnik Windows
- **TESTING.md** - Dokumentacja ogólna (Linux/Mac)

---

## 🎯 Czego oczekiwać?

Po uruchomieniu `quick_test.ps1`:

```
=========================================
  Turniej: Aether vs Stockfish
=========================================
Aether:           ...\aether-uci.exe
Stockfish:        stockfish.exe (poziom 5)
Liczba gier:      10
Time control:     10+0.1
=========================================

Rozpoczynanie turnieju...

[Cutechess wyświetla postęp...]

=========================================
Turniej zakończony!
=========================================
Wyniki zapisane w: quick_test.pgn

Analiza wyników:
Wygrane Aether:     4
Wygrane Stockfish:  5
Remisy:             1

Wynik Aether: 45.0%
```

**Dobry wynik:** 40-60% vs Stockfish poziom 5

---

## 💡 Wskazówki

1. **Zacznij od quick_test.bat** jeśli PowerShell sprawia problemy
2. **Użyj PowerShell** dla pełnej funkcjonalności i kolorów
3. **Zawsze buduj w release:** `cargo build --release` (nie `cargo build`)
4. **Zamknij i otwórz terminal** po dodaniu do PATH
5. **Uruchom jako Administrator** jeśli są problemy z uprawnieniami

---

## 🆘 Nadal nie działa?

1. Upewnij się że wszystkie 3 narzędzia są zainstalowane (Rust, cutechess, stockfish)
2. Sprawdź PATH: wszystkie muszą być dostępne z CMD/PowerShell
3. Zrestartuj komputer (czasem PATH wymaga restartu)
4. Użyj pełnych ścieżek w skryptach zamiast polegać na PATH
5. Sprawdź logi błędów w plikach PGN

---

## ✅ Checklist instalacji

- [ ] Rust zainstalowany (`cargo --version` działa)
- [ ] cutechess-cli zainstalowany (`cutechess-cli --version` działa)
- [ ] Stockfish zainstalowany (`stockfish.exe` uruchamia się)
- [ ] Aether zbudowany (`target\release\aether-uci.exe` istnieje)
- [ ] PowerShell ExecutionPolicy ustawiony
- [ ] `quick_test.ps1` lub `quick_test.bat` działa

---

## 🎮 Gotowe do gry!

Po instalacji możesz:
- ✅ Testować Aether vs Stockfish
- ✅ Symulować warunki Lichess
- ✅ Benchmarkować różne poziomy
- ✅ Analizować wyniki
- ✅ Debugować pojedyncze gry

**Powodzenia!** 🚀
