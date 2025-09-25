@echo off
setlocal EnableDelayedExpansion

REM QM Rust Agent Installer for Windows
REM Installs the Quine-McCluskey Boolean minimization agent for Claude Code

set "AGENT_NAME=qm-agent"
set "INSTALL_TYPE=local"

:parse_args
if "%~1"=="" goto main
if "%~1"=="-g" (
    set "INSTALL_TYPE=global"
    shift
    goto parse_args
)
if "%~1"=="--global" (
    set "INSTALL_TYPE=global"
    shift
    goto parse_args
)
if "%~1"=="-l" (
    set "INSTALL_TYPE=local"
    shift
    goto parse_args
)
if "%~1"=="--local" (
    set "INSTALL_TYPE=local"
    shift
    goto parse_args
)
if "%~1"=="-h" goto show_help
if "%~1"=="--help" goto show_help

echo Error: Unknown option %~1
goto show_help

:show_help
echo Usage: %~nx0 [OPTIONS]
echo.
echo Options:
echo   -g, --global    Install globally for all Claude Code projects
echo   -l, --local     Install for current project only (default)
echo   -h, --help      Show this help message
echo.
echo Examples:
echo   %~nx0              # Install locally
echo   %~nx0 --global     # Install globally
goto :eof

:print_header
echo ==================================
echo    QM Rust Agent Installer
echo ==================================
echo.
goto :eof

:check_dependencies
echo [INFO] Checking dependencies...

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust/Cargo is not installed. Please install Rust from https://rustup.rs/
    exit /b 1
)
echo [SUCCESS] Rust/Cargo found

where git >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Git is not installed. Please install Git first.
    exit /b 1
)
echo [SUCCESS] Git found
goto :eof

:build_binary
echo [INFO] Building QM Agent binary...

cargo build --release
if %errorlevel% neq 0 (
    echo [ERROR] Failed to build QM Agent binary
    exit /b 1
)
echo [SUCCESS] QM Agent binary built successfully
goto :eof

:install_agent
if "%INSTALL_TYPE%"=="global" (
    set "TARGET_DIR=%USERPROFILE%\.claude\agents"
    echo [INFO] Installing QM Agent globally for all projects...
) else (
    set "TARGET_DIR=.\.claude\agents"
    echo [INFO] Installing QM Agent for current project only...
)

if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"
echo [SUCCESS] Created directory: %TARGET_DIR%

if exist ".claude\agents\qm-agent.md" (
    copy ".claude\agents\qm-agent.md" "%TARGET_DIR%\" >nul
    echo [SUCCESS] Installed QM Agent configuration to %TARGET_DIR%
) else (
    echo [ERROR] QM Agent configuration not found. Are you in the correct directory?
    exit /b 1
)
goto :eof

:install_binary
if "%INSTALL_TYPE%"=="global" (
    echo [INFO] Installing binary globally...
    if exist "%USERPROFILE%\.cargo\bin" (
        copy "target\release\qm-agent.exe" "%USERPROFILE%\.cargo\bin\" >nul 2>nul
        if !errorlevel! equ 0 (
            echo [SUCCESS] Binary installed to %USERPROFILE%\.cargo\bin\
        ) else (
            echo [WARNING] Could not install to %USERPROFILE%\.cargo\bin, binary available at: target\release\qm-agent.exe
        )
    ) else (
        echo [WARNING] %USERPROFILE%\.cargo\bin not found, binary available at: target\release\qm-agent.exe
    )
) else (
    echo [INFO] Binary available at: target\release\qm-agent.exe
)
goto :eof

:main
call :print_header
call :check_dependencies
if %errorlevel% neq 0 exit /b %errorlevel%

call :build_binary
if %errorlevel% neq 0 exit /b %errorlevel%

call :install_agent
if %errorlevel% neq 0 exit /b %errorlevel%

call :install_binary

echo.
echo [SUCCESS] QM Rust Agent installed successfully!
echo.
echo [INFO] Usage examples:
echo   cargo run -- minimize -i "f(A,B,C) = Σ(1,3,7)"
echo   cargo run -- interactive
echo   cargo run -- examples
echo.
echo [INFO] Claude Code will automatically use this agent for Boolean minimization tasks.
echo.

if "%INSTALL_TYPE%"=="global" (
    echo [INFO] Agent installed globally - available in all Claude Code projects
) else (
    echo [INFO] Agent installed locally - available in current project only
)

goto :eof