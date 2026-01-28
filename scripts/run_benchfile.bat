@echo off
REM Run benchfile command on all algorithms and compare NPS by game phase
REM Usage: run_benchfile.bat [epd_file] [depth] [limit]
REM   epd_file - Path to EPD file (default: positions\opening\noob_3moves.epd)
REM   depth    - Search depth (default: 10)
REM   limit    - Max positions to test (default: 100)

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."
set "RESULTS_DIR=%PROJECT_DIR%\results"

REM Default parameters
if "%~1"=="" (
    set "EPD_FILE=%PROJECT_DIR%\positions\opening\noob_3moves.epd"
) else (
    set "EPD_FILE=%~1"
)

if "%~2"=="" (
    set "DEPTH=10"
) else (
    set "DEPTH=%~2"
)

if "%~3"=="" (
    set "LIMIT=100"
) else (
    set "LIMIT=%~3"
)

REM Build engine
echo Building engine...
cargo build --release --package aether 2>nul

set "AETHER_BIN=%PROJECT_DIR%\target\release\aether.exe"
if not exist "%AETHER_BIN%" (
    echo Error: Engine binary not found
    exit /b 1
)

if not exist "%EPD_FILE%" (
    echo Error: EPD file not found: %EPD_FILE%
    echo Run .\scripts\download_positions.bat first
    exit /b 1
)

if not exist "%RESULTS_DIR%" mkdir "%RESULTS_DIR%"

for /f "tokens=2-4 delims=/ " %%a in ('date /t') do set "DATESTAMP=%%c%%a%%b"
for /f "tokens=1-2 delims=: " %%a in ('time /t') do set "TIMESTAMP=%DATESTAMP%_%%a%%b"

echo.
echo === Benchfile Comparison ===
echo File: %EPD_FILE%
echo Depth: %DEPTH%
echo Limit: %LIMIT% positions
echo.

REM Algorithms to test
set ALGORITHMS=FullAlphaBeta PureAlphaBeta Mtdf NegaScout MCTS

for %%a in (%ALGORITHMS%) do (
    echo.
    echo =====================================================
    echo Algorithm: %%a
    echo =====================================================

    set "OUTPUT_FILE=%RESULTS_DIR%\benchfile_%%a_%TIMESTAMP%.txt"

    REM Run benchfile command via UCI
    (
        echo setoption name Algorithm value %%a
        echo benchfile %EPD_FILE% %DEPTH% %LIMIT%
        echo quit
    ) | "%AETHER_BIN%" > "!OUTPUT_FILE!"

    type "!OUTPUT_FILE!"
    echo.
    echo Results saved to: !OUTPUT_FILE!
)

echo.
echo === Comparison Complete ===
echo Results saved to: %RESULTS_DIR%\

endlocal