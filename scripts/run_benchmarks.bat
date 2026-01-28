@echo off
setlocal enabledelayedexpansion

REM === Konfiguracja ===
set TIME_MS=5000
set LIMIT=1000
set THREADS=14

REM === Sciezki ===
set "SCRIPT_DIR=%~dp0"
pushd "%SCRIPT_DIR%.."
set "PROJECT_DIR=%CD%"
popd

echo ==================================
echo Aether Benchmark Comparison Suite
echo ==================================
echo.
echo Settings:
echo   Time limit: %TIME_MS%ms
echo   Limit: %LIMIT% positions per file
echo   Threads: %THREADS%
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
if not exist "results" mkdir "results"

REM === Timestamp ===
for /f %%i in ('powershell -command "Get-Date -Format yyyyMMdd_HHmmss"') do set TIMESTAMP=%%i
set "COMBINED_CSV=results\comparison_%TIMESTAMP%.csv"

REM === Sprawdz katalog pozycji ===
if not exist "positions" (
    echo Positions directory not found
    echo Run .\scripts\download_positions.bat first
    exit /b 1
)

REM === Pliki EPD ===
set FIRST_FILE=1
set EPD_COUNT=0
for /r "positions" %%F in (*.epd) do set /a EPD_COUNT+=1
if %EPD_COUNT%==0 (
    echo No EPD files found in positions
    exit /b 1
)

REM === Plik tymczasowy na komendy ===
set "CMD_FILE=%TEMP%\aether_cmd_%TIMESTAMP%.txt"

for /r "positions" %%F in (*.epd) do (
    set "EPD_FULL=%%F"
    set "BASENAME=%%~nF"
    set "OUTPUT_CSV=results\!BASENAME!_%TIMESTAMP%.csv"

    REM Konwertuj sciezke absolutna na wzgledna
    set "EPD_REL=!EPD_FULL:%PROJECT_DIR%\=!"

    echo Processing: !BASENAME!
    echo   Input:  !EPD_REL!
    echo   Output: !OUTPUT_CSV!

    REM Zapisz komendy do pliku
    echo benchexport !EPD_REL! !OUTPUT_CSV! %TIME_MS% %LIMIT% %THREADS%> "!CMD_FILE!"
    echo quit>> "!CMD_FILE!"

    REM Uruchom silnik z pliku komend
    "target\release\aether.exe" < "!CMD_FILE!"

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

REM Usun plik tymczasowy
if exist "%CMD_FILE%" del "%CMD_FILE%"

echo ==================================
echo Combined results: %COMBINED_CSV%
echo ==================================
echo.
echo Benchmark complete!
pause