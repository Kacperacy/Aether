@echo off
REM Run a tournament between all chess engine algorithms using fastchess
REM Usage: run_tournament.bat [tc] [rounds]
REM   tc     - Time control in seconds+increment format (default: 10+0.1)
REM   rounds - Number of rounds per pairing (default: 50)

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."
set "RESULTS_DIR=%PROJECT_DIR%\results"

REM Default parameters
if "%~1"=="" (set "TC=10+0.1") else (set "TC=%~1")
if "%~2"=="" (set "ROUNDS=50") else (set "ROUNDS=%~2")
if "%~3"=="" (set "CONCURRENCY=4") else (set "CONCURRENCY=%~3")

REM Check if fastchess is available
where fastchess >nul 2>nul
if errorlevel 1 (
    echo Error: fastchess is not installed or not in PATH
    echo Install from: https://github.com/Disservin/fastchess
    exit /b 1
)

REM Build all binaries in release mode
echo Building engine binaries...
cargo build --release --package aether 2>nul

REM Check if binaries exist
set "AETHER_BIN=%PROJECT_DIR%\target\release\aether.exe"
if not exist "%AETHER_BIN%" (
    echo Error: Engine binary not found at %AETHER_BIN%
    exit /b 1
)

if not exist "%RESULTS_DIR%" mkdir "%RESULTS_DIR%"

REM Generate timestamp for results
for /f "tokens=2-4 delims=/ " %%a in ('date /t') do set "DATESTAMP=%%c%%a%%b"
for /f "tokens=1-2 delims=: " %%a in ('time /t') do set "TIMESTAMP=%DATESTAMP%_%%a%%b"
set "PGN_FILE=%RESULTS_DIR%\tournament_%TIMESTAMP%.pgn"

echo.
echo === Aether Chess Engine Tournament ===
echo Time control: %TC%
echo Rounds per pairing: %ROUNDS%
echo Concurrency: %CONCURRENCY%
echo Results: %PGN_FILE%
echo.

REM Run tournament with all algorithms
fastchess ^
    -engine cmd="%AETHER_BIN%" name=FullAlphaBeta option.Algorithm=FullAlphaBeta ^
    -engine cmd="%AETHER_BIN%" name=PureAlphaBeta option.Algorithm=PureAlphaBeta ^
    -engine cmd="%AETHER_BIN%" name=MTDf option.Algorithm=Mtdf ^
    -engine cmd="%AETHER_BIN%" name=NegaScout option.Algorithm=NegaScout ^
    -engine cmd="%AETHER_BIN%" name=MCTS option.Algorithm=MCTS ^
    -each tc="%TC%" ^
    -rounds %ROUNDS% ^
    -games 2 ^
    -repeat ^
    -recover ^
    -concurrency %CONCURRENCY% ^
    -pgnout "%PGN_FILE%"

echo.
echo Tournament complete!
echo Results saved to: %PGN_FILE%

endlocal