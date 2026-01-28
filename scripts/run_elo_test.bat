@echo off
setlocal enabledelayedexpansion

REM === Konfiguracja ===
set TC=10+0.1
set ROUNDS=50
set CONCURRENCY=7

REM === Sciezki ===
set "SCRIPT_DIR=%~dp0"
pushd "%SCRIPT_DIR%.."
set "PROJECT_DIR=%CD%"
popd
set "RESULTS_DIR=%PROJECT_DIR%\results"

REM === Sprawdz Stockfish ===
where stockfish >nul 2>nul
if errorlevel 1 (
    echo Error: stockfish is not installed or not in PATH
    echo Download from: https://stockfishchess.org/download/
    exit /b 1
)

REM === Sprawdz fastchess ===
where fastchess >nul 2>nul
if errorlevel 1 (
    echo Error: fastchess is not installed or not in PATH
    echo Install from: https://github.com/Disservin/fastchess
    exit /b 1
)

echo ==================================
echo Aether ELO Rating Test
echo ==================================
echo.
echo Settings:
echo   Time control: %TC%
echo   Rounds: %ROUNDS% per opponent
echo   Concurrency: %CONCURRENCY%
echo.

REM === Kompilacja ===
echo Building engine in release mode...
cd /d "%PROJECT_DIR%"
cargo build --release --package aether
if errorlevel 1 (
    echo Build failed!
    exit /b 1
)
echo Build successful!
echo.

set "AETHER_BIN=%PROJECT_DIR%\target\release\aether.exe"
if not exist "%AETHER_BIN%" (
    echo Error: Engine binary not found
    exit /b 1
)

if not exist "%RESULTS_DIR%" mkdir "%RESULTS_DIR%"

REM === Timestamp ===
for /f %%i in ('powershell -command "Get-Date -Format yyyyMMdd_HHmmss"') do set TIMESTAMP=%%i
set "PGN_FILE=%RESULTS_DIR%\elo_test_%TIMESTAMP%.pgn"

echo === Starting ELO Test Tournament ===
echo Aether vs Stockfish (ELO 1350, 1800, 2200)
echo Output: %PGN_FILE%
echo.

set "OPENINGS=%PROJECT_DIR%\positions\UHO_4060_v4.epd"

if not exist "%OPENINGS%" (
    echo Error: Opening book not found: %OPENINGS%
    exit /b 1
)

fastchess ^
    -engine cmd="%AETHER_BIN%" name=FullAlphaBeta option.Algorithm=FullAlphaBeta ^
    -engine cmd="%AETHER_BIN%" name=PureAlphaBeta option.Algorithm=PureAlphaBeta ^
    -engine cmd="%AETHER_BIN%" name=MTDf option.Algorithm=Mtdf ^
    -engine cmd="%AETHER_BIN%" name=NegaScout option.Algorithm=NegaScout ^
    -engine cmd="%AETHER_BIN%" name=MCTS option.Algorithm=MCTS ^
    -engine cmd="%AETHER_BIN%" name=ClassicMCTS option.Algorithm=ClassicMCTS ^
    -engine cmd="stockfish" name="SF_1350" option.UCI_LimitStrength=true option.UCI_Elo=1350 ^
    -engine cmd="stockfish" name="SF_1800" option.UCI_LimitStrength=true option.UCI_Elo=1800 ^
    -engine cmd="stockfish" name="SF_2200" option.UCI_LimitStrength=true option.UCI_Elo=2200 ^
    -openings file="%OPENINGS%" format=epd order=random ^
    -each tc=%TC% ^
    -rounds %ROUNDS% ^
    -games 2 ^
    -repeat ^
    -recover ^
    -concurrency %CONCURRENCY% ^
    -pgnout file="%PGN_FILE%"

echo.
echo === Tournament Complete ===
echo Results saved to: %PGN_FILE%
echo.
pause