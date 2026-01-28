@echo off
REM Run tests by game phase (opening, middlegame, endgame) using fastchess
REM Requires position files downloaded via download_positions.bat
REM
REM Usage: run_phase_tests.bat [algorithm1] [algorithm2] [tc] [rounds]
REM   algorithm1 - First algorithm (default: FullAlphaBeta)
REM   algorithm2 - Second algorithm (default: MCTS)
REM   tc         - Time control (default: 10+0.1)
REM   rounds     - Rounds per phase (default: 50)

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."
set "POSITIONS_DIR=%PROJECT_DIR%\positions"
set "RESULTS_DIR=%PROJECT_DIR%\results"

REM Default parameters
if "%~1"=="" (set "ALGO1=FullAlphaBeta") else (set "ALGO1=%~1")
if "%~2"=="" (set "ALGO2=MCTS") else (set "ALGO2=%~2")
if "%~3"=="" (set "TC=10+0.1") else (set "TC=%~3")
if "%~4"=="" (set "ROUNDS=50") else (set "ROUNDS=%~4")
if "%~5"=="" (set "CONCURRENCY=4") else (set "CONCURRENCY=%~5")

REM Check if fastchess is available
where fastchess >nul 2>nul
if errorlevel 1 (
    echo Error: fastchess is not installed or not in PATH
    echo Install from: https://github.com/Disservin/fastchess
    exit /b 1
)

REM Build engine
echo Building engine...
cargo build --release --package aether 2>nul

set "AETHER_BIN=%PROJECT_DIR%\target\release\aether.exe"
if not exist "%AETHER_BIN%" (
    echo Error: Engine binary not found
    exit /b 1
)

if not exist "%RESULTS_DIR%" mkdir "%RESULTS_DIR%"

REM Generate timestamp
for /f "tokens=2-4 delims=/ " %%a in ('date /t') do set "DATESTAMP=%%c%%a%%b"
for /f "tokens=1-2 delims=: " %%a in ('time /t') do set "TIMESTAMP=%DATESTAMP%_%%a%%b"

echo.
echo === Phase-based Testing: %ALGO1% vs %ALGO2% ===
echo Time control: %TC%
echo Rounds per phase: %ROUNDS%
echo.

REM Opening test
echo === OPENING POSITIONS ===
set "OPENING_EPD=%POSITIONS_DIR%\opening\noob_3moves.epd"
if not exist "%OPENING_EPD%" set "OPENING_EPD=%POSITIONS_DIR%\opening\2moves_v1.epd"

if exist "%OPENING_EPD%" (
    set "PGN_FILE=%RESULTS_DIR%\opening_%ALGO1%_vs_%ALGO2%_%TIMESTAMP%.pgn"
    echo --- opening ---
    echo Positions: %OPENING_EPD%
    echo Output: !PGN_FILE!
    echo.

    fastchess ^
        -engine cmd="%AETHER_BIN%" name="%ALGO1%" option.Algorithm="%ALGO1%" ^
        -engine cmd="%AETHER_BIN%" name="%ALGO2%" option.Algorithm="%ALGO2%" ^
        -openings file="%OPENING_EPD%" format=epd order=random ^
        -each tc="%TC%" ^
        -rounds %ROUNDS% ^
        -games 2 ^
        -repeat ^
        -recover ^
        -concurrency %CONCURRENCY% ^
        -pgnout "!PGN_FILE!"
    echo.
) else (
    echo Skipping opening: Position file not found
    echo Run .\scripts\download_positions.bat first
)

REM Middlegame test
echo === MIDDLEGAME POSITIONS ===
set "MIDDLEGAME_EPD=%POSITIONS_DIR%\middlegame\Drawkiller_balanced_big.epd"

if exist "%MIDDLEGAME_EPD%" (
    set "PGN_FILE=%RESULTS_DIR%\middlegame_%ALGO1%_vs_%ALGO2%_%TIMESTAMP%.pgn"
    echo --- middlegame ---
    echo Positions: %MIDDLEGAME_EPD%
    echo Output: !PGN_FILE!
    echo.

    fastchess ^
        -engine cmd="%AETHER_BIN%" name="%ALGO1%" option.Algorithm="%ALGO1%" ^
        -engine cmd="%AETHER_BIN%" name="%ALGO2%" option.Algorithm="%ALGO2%" ^
        -openings file="%MIDDLEGAME_EPD%" format=epd order=random ^
        -each tc="%TC%" ^
        -rounds %ROUNDS% ^
        -games 2 ^
        -repeat ^
        -recover ^
        -concurrency %CONCURRENCY% ^
        -pgnout "!PGN_FILE!"
    echo.
) else (
    echo Skipping middlegame: Position file not found
    echo Run .\scripts\download_positions.bat first
)

REM Endgame test - longer time control
echo === ENDGAME POSITIONS ===
set "ENDGAME_EPD=%POSITIONS_DIR%\endgame\endgames.epd"

if exist "%ENDGAME_EPD%" (
    set "PGN_FILE=%RESULTS_DIR%\endgame_%ALGO1%_vs_%ALGO2%_%TIMESTAMP%.pgn"
    echo --- endgame ---
    echo Positions: %ENDGAME_EPD%
    echo Output: !PGN_FILE!
    echo.

    fastchess ^
        -engine cmd="%AETHER_BIN%" name="%ALGO1%" option.Algorithm="%ALGO1%" ^
        -engine cmd="%AETHER_BIN%" name="%ALGO2%" option.Algorithm="%ALGO2%" ^
        -openings file="%ENDGAME_EPD%" format=epd order=random ^
        -each tc="30+0.3" ^
        -rounds %ROUNDS% ^
        -games 2 ^
        -repeat ^
        -recover ^
        -concurrency %CONCURRENCY% ^
        -pgnout "!PGN_FILE!"
    echo.
) else (
    echo Skipping endgame: Position file not found
    echo Run .\scripts\download_positions.bat first
)

echo.
echo === Phase Testing Complete ===
echo Results saved to: %RESULTS_DIR%\
echo.
echo To analyze results:
echo   - Use fastchess output for Elo estimates
echo   - PGN files can be analyzed with pgn-extract or python-chess

endlocal