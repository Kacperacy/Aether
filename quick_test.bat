@echo off
REM Szybki test - 10 gier vs Stockfish poziom 5 (Windows Batch)

echo ========================================
echo  Szybki test Aether vs Stockfish
echo ========================================
echo.

set AETHER_PATH=%~dp0target\release\aether-uci.exe
set OUTPUT=quick_test.pgn

if not exist "%AETHER_PATH%" (
    echo BLAD: Nie znaleziono Aether w: %AETHER_PATH%
    echo Zbuduj silnik: cargo build --release
    pause
    exit /b 1
)

cutechess-cli ^
    -engine cmd="%AETHER_PATH%" name="Aether" option."Move Overhead"=100 ^
    -engine cmd="stockfish.exe" name="Stockfish-L5" option."Skill Level"=5 ^
    -each tc="10+0.1" ^
    -rounds 5 ^
    -games 2 ^
    -repeat ^
    -concurrency 2 ^
    -pgnout "%OUTPUT%" ^
    -ratinginterval 10 ^
    -recover

echo.
echo ========================================
echo  Test zakonczony!
echo ========================================
echo Wyniki w pliku: %OUTPUT%
echo.
echo Uruchom: analyze_results.ps1 %OUTPUT%
pause
