@echo off
REM Script to benchmark MAX_16_BITS = true vs false (Windows version)
REM This automates the process of running benchmarks for both configurations

setlocal enabledelayedexpansion

set CLASSIC_RS=src\qm\classic.rs
set BACKUP_FILE=%CLASSIC_RS%.backup
set RESULTS_DIR=benches\results

echo === MAX_16_BITS Performance Benchmark ===
echo.

REM Create results directory
if not exist "%RESULTS_DIR%" mkdir "%RESULTS_DIR%"

REM Backup original file
echo Creating backup of %CLASSIC_RS%
copy "%CLASSIC_RS%" "%BACKUP_FILE%" >nul

REM Function to set MAX_16_BITS for 32-bit mode
:set_32bit
echo Setting MAX_16_BITS = false (32-bit mode)
powershell -Command "(Get-Content '%CLASSIC_RS%') -replace 'pub const MAX_16_BITS: bool = .*', 'pub const MAX_16_BITS: bool = false;' | Set-Content '%CLASSIC_RS%'"
goto :eof

REM Function to set MAX_16_BITS for 16-bit mode
:set_16bit
echo Setting MAX_16_BITS = true (16-bit mode)
powershell -Command "(Get-Content '%CLASSIC_RS%') -replace 'pub const MAX_16_BITS: bool = .*', 'pub const MAX_16_BITS: bool = true;' | Set-Content '%CLASSIC_RS%'"
goto :eof

REM Step 1: Benchmark 32-bit mode
call :set_32bit
echo.
echo Running benchmarks with 32-bit mode...
echo This may take 5-10 minutes...
cargo bench --bench max_16_bits_bench -- --save-baseline 32bit > "%RESULTS_DIR%\raw_32bit.txt" 2>&1
if errorlevel 1 (
    echo ERROR: Benchmark failed
    goto restore
)
echo [OK] Benchmarks complete for 32-bit mode
echo.

REM Step 2: Benchmark 16-bit mode
call :set_16bit
echo.
echo Running benchmarks with 16-bit mode...
echo This may take 5-10 minutes...
cargo bench --bench max_16_bits_bench -- --save-baseline 16bit > "%RESULTS_DIR%\raw_16bit.txt" 2>&1
if errorlevel 1 (
    echo ERROR: Benchmark failed
    goto restore
)
echo [OK] Benchmarks complete for 16-bit mode
echo.

:restore
REM Restore original file
echo Restoring original %CLASSIC_RS%
copy /Y "%BACKUP_FILE%" "%CLASSIC_RS%" >nul
del "%BACKUP_FILE%"

REM Compare results using critcmp if available
echo.
echo === Comparing Results ===
where critcmp >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo Using critcmp for comparison:
    critcmp 32bit 16bit > "%RESULTS_DIR%\comparison.txt"
    type "%RESULTS_DIR%\comparison.txt"
) else (
    echo Note: Install critcmp for detailed comparison:
    echo   cargo install critcmp
    echo.
    echo Then run:
    echo   critcmp 32bit 16bit
)

echo.
echo === Benchmark Complete ===
echo Results saved in: %RESULTS_DIR%\
echo - raw_32bit.txt: Full output for 32-bit mode
echo - raw_16bit.txt: Full output for 16-bit mode
echo - comparison.txt: Side-by-side comparison (if critcmp installed)
echo.
echo Criterion HTML reports available at:
echo   target\criterion\report\index.html
echo.
pause
