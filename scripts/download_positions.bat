@echo off
REM Download test positions from Stockfish Books repository
REM https://github.com/official-stockfish/books

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PROJECT_DIR=%SCRIPT_DIR%.."
set "POSITIONS_DIR=%PROJECT_DIR%\positions"

set "STOCKFISH_BOOKS_RAW=https://raw.githubusercontent.com/official-stockfish/books/master"

echo Downloading test positions from Stockfish Books...
echo Target directory: %POSITIONS_DIR%
echo.

if not exist "%POSITIONS_DIR%\opening" mkdir "%POSITIONS_DIR%\opening"
if not exist "%POSITIONS_DIR%\middlegame" mkdir "%POSITIONS_DIR%\middlegame"
if not exist "%POSITIONS_DIR%\endgame" mkdir "%POSITIONS_DIR%\endgame"

REM Opening positions (after 3 moves from start)
echo.
echo === Opening Positions ===
echo Downloading noob_3moves.epd...
curl -L --progress-bar "%STOCKFISH_BOOKS_RAW%/noob_3moves.epd.zip" -o "%TEMP%\noob_3moves.epd.zip"
if exist "%TEMP%\noob_3moves.epd.zip" (
    powershell -command "Expand-Archive -Force '%TEMP%\noob_3moves.epd.zip' '%POSITIONS_DIR%\opening\'"
    del "%TEMP%\noob_3moves.epd.zip"
    echo   -^> Saved to %POSITIONS_DIR%\opening\
)

echo Downloading 2moves_v1.epd ^(alternative^)...
curl -L --progress-bar "%STOCKFISH_BOOKS_RAW%/2moves_v1.epd.zip" -o "%TEMP%\2moves_v1.epd.zip"
if exist "%TEMP%\2moves_v1.epd.zip" (
    powershell -command "Expand-Archive -Force '%TEMP%\2moves_v1.epd.zip' '%POSITIONS_DIR%\opening\'"
    del "%TEMP%\2moves_v1.epd.zip"
    echo   -^> Saved to %POSITIONS_DIR%\opening\
)

REM Endgame positions
echo.
echo === Endgame Positions ===
echo Downloading endgames.epd...
curl -L --progress-bar "%STOCKFISH_BOOKS_RAW%/endgames.epd.zip" -o "%TEMP%\endgames.epd.zip"
if exist "%TEMP%\endgames.epd.zip" (
    powershell -command "Expand-Archive -Force '%TEMP%\endgames.epd.zip' '%POSITIONS_DIR%\endgame\'"
    del "%TEMP%\endgames.epd.zip"
    echo   -^> Saved to %POSITIONS_DIR%\endgame\
)

REM Middlegame positions
echo.
echo === Middlegame Positions ===
echo Note: Full middlegame books are very large ^(100MB+^).
echo Downloading smaller balanced set...
echo Downloading Drawkiller_balanced_big.epd...
curl -L --progress-bar "%STOCKFISH_BOOKS_RAW%/Drawkiller_balanced_big.epd.zip" -o "%TEMP%\drawkiller.epd.zip"
if exist "%TEMP%\drawkiller.epd.zip" (
    powershell -command "Expand-Archive -Force '%TEMP%\drawkiller.epd.zip' '%POSITIONS_DIR%\middlegame\'"
    del "%TEMP%\drawkiller.epd.zip"
    echo   -^> Saved to %POSITIONS_DIR%\middlegame\
)

REM Summary
echo.
echo === Download Complete ===
echo.
echo Files downloaded:
for /r "%POSITIONS_DIR%" %%f in (*.epd) do (
    for /f %%c in ('find /c /v "" ^< "%%f"') do echo   %%f: %%c positions
)

echo.
echo Usage with fastchess:
echo   fastchess -openings file=positions/opening/noob_3moves.epd format=epd ...
echo.
echo Usage with benchfile command:
echo   .\target\release\aether.exe
echo   ^> benchfile positions/opening/noob_3moves.epd 10
echo.

endlocal