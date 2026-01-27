@echo off
setlocal enabledelayedexpansion

REM === Konfiguracja ===
set TIME_MS=1000
set LIMIT=500

REM === Sciezki ===
set SCRIPT_DIR=%~dp0
set PROJECT_DIR=%SCRIPT_DIR%..
set RESULTS_DIR=%PROJECT_DIR%\results
set POSITIONS_DIR=%PROJECT_DIR%\positions

echo ==================================
echo Aether Benchmark Comparison Suite
echo ==================================
echo.
echo Settings:
echo   Time limit: %TIME_MS%ms
if defined LIMIT (echo   Limit: %LIMIT% positions) else (echo   Limit: All positions)
echo.

REM === Kompilacja ===
echo Building engine in release mode...
cd /d "%PROJECT_DIR%"
cargo build --release
if errorlevel 1 (
    echo Build failed!
    exit /b 1
)
echo Build successful!
echo.

REM === Katalog wynikow ===
if not exist "%RESULTS_DIR%" mkdir "%RESULTS_DIR%"

REM === Timestamp ===
for /f "tokens=2 delims==" %%I in ('wmic os get localdatetime /value') do set datetime=%%I
set TIMESTAMP=%datetime:~0,8%_%datetime:~8,6%
set COMBINED_CSV=%RESULTS_DIR%\comparison_%TIMESTAMP%.csv

REM === Pliki EPD ===
set FIRST_FILE=1
for /r "%POSITIONS_DIR%" %%F in (*.epd) do (
    set "EPD_FILE=%%F"
    set "BASENAME=%%~nF"
    set "OUTPUT_CSV=%RESULTS_DIR%\!BASENAME!_%TIMESTAMP%.csv"

    echo Processing: !BASENAME!
    echo benchexport "!EPD_FILE!" "!OUTPUT_CSV!" %TIME_MS% %LIMIT% | "%PROJECT_DIR%\target\release\aether.exe"

    if exist "!OUTPUT_CSV!" (
        echo Results saved to: !OUTPUT_CSV!

        if !FIRST_FILE!==1 (
            copy "!OUTPUT_CSV!" "%COMBINED_CSV%" >nul
            set FIRST_FILE=0
        ) else (
            for /f "skip=1 delims=" %%L in ('type "!OUTPUT_CSV!"') do echo %%L>>"%COMBINED_CSV%"
        )
    ) else (
        echo Failed to generate results
    )
    echo.
)

echo ==================================
echo Combined results: %COMBINED_CSV%
echo ==================================
echo.
echo Benchmark complete!
pause
