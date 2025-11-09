@echo off
REM Uniwersalny turniej Aether vs Stockfish (Windows Batch)
REM Użycie: run_tournament.bat [level] [games] [time_control]

setlocal enabledelayedexpansion

set LEVEL=%1
set GAMES=%2
set TIME_CONTROL=%3

if "%LEVEL%"=="" set LEVEL=5
if "%GAMES%"=="" set GAMES=100
if "%TIME_CONTROL%"=="" set TIME_CONTROL=40/60+0.6

set AETHER_PATH=%~dp0target\release\aether.exe
set ROUNDS=%GAMES%
set /a ROUNDS=%GAMES%/2

for /f "tokens=2-4 delims=/ " %%a in ('date /t') do (set DATE=%%c%%b%%a)
for /f "tokens=1-2 delims=/:" %%a in ('time /t') do (set TIME=%%a%%b)
set TIMESTAMP=%DATE%_%TIME%
set OUTPUT=results_sf%LEVEL%_%TIMESTAMP%.pgn

echo =========================================
echo   Turniej: Aether vs Stockfish
echo =========================================
echo Aether:           %AETHER_PATH%
echo Stockfish:        stockfish.exe (poziom %LEVEL%)
echo Liczba gier:      %GAMES%
echo Time control:     %TIME_CONTROL%
echo Plik wynikowy:    %OUTPUT%
echo =========================================
echo.

if not exist "%AETHER_PATH%" (
    echo BLAD: Nie znaleziono Aether
    echo Zbuduj: cargo build --release
    pause
    exit /b 1
)

cutechess-cli ^
    -engine cmd="%AETHER_PATH%" name="Aether" proto=uci option."Move Overhead"=100 ^
    -engine cmd="stockfish.exe" name="Stockfish-L%LEVEL%" proto=uci option."Skill Level"=%LEVEL% ^
    -each tc="%TIME_CONTROL%" ^
    -openings file="%~dp0openings.epd" format=epd order=random ^
    -rounds %ROUNDS% ^
    -games 2 ^
    -repeat ^
    -pgnout "%OUTPUT%" ^
    -ratinginterval 10 ^
    -recover

echo.
echo =========================================
echo  Turniej zakonczony!
echo =========================================
echo Wyniki zapisane w: %OUTPUT%
echo.
pause
