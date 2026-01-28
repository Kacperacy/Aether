@echo off
REM Run benchmark comparison across all algorithms on test positions
REM Usage: run_comparison.bat [time_ms] [limit]
REM   time_ms - Time limit in milliseconds (default: 1000)
REM   limit - Max positions per file (default: all)

setlocal enabledelayedexpansion

if "%~1"=="" (
    set "TIME_MS=1000"
) else (
    set "TIME_MS=%~1"
)

set "LIMIT=%~2"

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."
set "RESULTS_DIR=%PROJECT_DIR%\results"
set "POSITIONS_DIR=%PROJECT_DIR%\positions"

echo ==================================
echo Aether Benchmark Comparison Suite
echo ==================================
echo.
echo Settings:
echo   Time limit: %TIME_MS%ms
if defined LIMIT (
    echo   Limit: %LIMIT% positions per file
) else (
    echo   Limit: All positions
)
echo.

REM Build the engine
echo Building engine in release mode...
pushd "%PROJECT_DIR%"
cargo build --release 2>nul
if errorlevel 1 (
    echo Build failed!
    popd
    exit /b 1
)
popd
echo Build successful!
echo.

REM Create results directory
if not exist "%RESULTS_DIR%" mkdir "%RESULTS_DIR%"

REM Check positions directory
if not exist "%POSITIONS_DIR%" (
    echo Positions directory not found: %POSITIONS_DIR%
    echo Please create test positions or run .\scripts\download_positions.bat
    exit /b 1
)

REM Find all EPD files
set "EPD_COUNT=0"
for /r "%POSITIONS_DIR%" %%f in (*.epd) do (
    set /a EPD_COUNT+=1
)

if %EPD_COUNT%==0 (
    echo No EPD files found in %POSITIONS_DIR%
    exit /b 1
)

for /f "tokens=2-4 delims=/ " %%a in ('date /t') do set "DATESTAMP=%%c%%a%%b"
for /f "tokens=1-2 delims=: " %%a in ('time /t') do set "TIMESTAMP=%DATESTAMP%_%%a%%b"
set "COMBINED_CSV=%RESULTS_DIR%\comparison_%TIMESTAMP%.csv"

echo Found EPD files:
for /r "%POSITIONS_DIR%" %%f in (*.epd) do (
    echo   - %%~nxf
)
echo.

REM Process each EPD file
set "FIRST_FILE=1"
for /r "%POSITIONS_DIR%" %%f in (*.epd) do (
    set "EPD_FILE=%%f"
    set "BASENAME=%%~nf"
    set "OUTPUT_CSV=%RESULTS_DIR%\!BASENAME!_%TIMESTAMP%.csv"

    echo Processing: !BASENAME!

    set "CMD=benchexport !EPD_FILE! !OUTPUT_CSV! %TIME_MS%"

    REM Run the benchmark
    echo !CMD! | "%PROJECT_DIR%\target\release\aether.exe" 2>&1

    if exist "!OUTPUT_CSV!" (
        echo Results saved to: !OUTPUT_CSV!

        REM Append to combined CSV
        if !FIRST_FILE!==1 (
            copy "!OUTPUT_CSV!" "%COMBINED_CSV%" >nul
            set "FIRST_FILE=0"
        ) else (
            for /f "skip=1 delims=" %%l in ('type "!OUTPUT_CSV!"') do (
                echo %%l>>"%COMBINED_CSV%"
            )
        )
    ) else (
        echo Failed to generate results for !BASENAME!
    )

    echo.
)

REM Generate summary
if exist "%COMBINED_CSV%" (
    echo ==================================
    echo Combined Results Summary
    echo ==================================
    echo.
    echo Output file: %COMBINED_CSV%
    echo.
    echo Benchmark comparison complete!
) else (
    echo No combined results generated
)

endlocal