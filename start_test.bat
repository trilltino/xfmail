@echo off
REM Full Debugging Test Runner for XFMail
REM Runs backend server and two client instances with complete tracing

setlocal enabledelayedexpansion

set RUST_LOG=debug,xfmail=debug
set DEV_AUTH_BYPASS=1
set SERVER_PORT=3000
set CLIENT_API_URL=http://127.0.0.1:3000

REM User IDs for the two test clients
set CLIENT1_USER_ID=00000000-0000-0000-0000-000000000001
set CLIENT2_USER_ID=00000000-0000-0000-0000-000000000002

echo.
echo ========================================
echo XFMail Full Debug Test Suite
echo ========================================
echo RUST_LOG=%RUST_LOG%
echo DEV_AUTH_BYPASS=%DEV_AUTH_BYPASS%
echo SERVER_PORT=%SERVER_PORT%
echo CLIENT1_USER_ID=%CLIENT1_USER_ID%
echo CLIENT2_USER_ID=%CLIENT2_USER_ID%
echo.

echo [0/5] Cleaning up any existing processes...
taskkill /F /IM xfcollab-server.exe 2>nul
taskkill /F /IM egui_app.exe 2>nul

echo Waiting for processes to terminate...
timeout /t 2 /nobreak >nul

echo [1/6] Building backend server with SSR feature...
call cargo build --release --bin xfcollab-server --features ssr
if %ERRORLEVEL% neq 0 (
    echo ERROR: Backend build failed
    pause
    exit /b 1
)

echo [2/6] Building client application...
call cargo build --release --bin egui_app
if %ERRORLEVEL% neq 0 (
    echo ERROR: Client build failed
    pause
    exit /b 1
)

echo [3/6] Ensuring database exists...
REM Try to create database using psql (if available) or show manual instructions
echo Attempting to create database 'xfmail'...
psql -U postgres -h localhost -p 5433 -f CREATE_DATABASE.sql 2>nul
if %ERRORLEVEL% neq 0 (
    echo ⚠️  Could not auto-create database. Please ensure:
    echo    1. PostgreSQL is running on port 5433
    echo    2. Database 'xfmail' exists
    echo    3. User 'postgres' can connect
    echo.
    echo You can create the database manually via pgAdmin or run:
    echo psql -U postgres -h localhost -p 5433 -f CREATE_DATABASE.sql
    echo.
)

echo [4/6] Starting backend server on port %SERVER_PORT%...
set RUST_LOG=%RUST_LOG%
set DEV_AUTH_BYPASS=%DEV_AUTH_BYPASS%
set SERVER_PORT=%SERVER_PORT%
start "XFMail Backend Server" cmd /c "cd /d %cd% && cargo run --release --bin xfcollab-server --features ssr"

REM Wait for server to start
echo Waiting 3 seconds for server to initialize...
timeout /t 3 /nobreak

echo [5/6] Launching Client 1 (User: %CLIENT1_USER_ID%)...
set DEV_USER_ID=%CLIENT1_USER_ID%
set CLIENT_API_URL=%CLIENT_API_URL%
start "XFMail Client 1" cmd /c "cd /d %cd% && cargo run --release --bin egui_app"

REM Wait between client launches
echo Waiting 2 seconds before launching Client 2...
timeout /t 2 /nobreak

echo [6/6] Launching Client 2 (User: %CLIENT2_USER_ID%)...
set DEV_USER_ID=%CLIENT2_USER_ID%
set CLIENT_API_URL=%CLIENT_API_URL%
start "XFMail Client 2" cmd /c "cd /d %cd% && cargo run --release --bin egui_app"

echo.
echo ========================================
echo All processes started!
echo ========================================
echo Backend: http://127.0.0.1:3000
echo Client 1: User %CLIENT1_USER_ID%
echo Client 2: User %CLIENT2_USER_ID%
echo.
echo Database Status:
if exist .env (
    for /f "tokens=2 delims==" %%i in (.env) do if not "%%i"=="" echo   DATABASE_URL configured
)
echo.
echo To test messaging:
echo 1. Create a conversation in both clients
echo 2. Add each other as participants
echo 3. Send messages and watch them sync via Braid HTTP
echo.
echo If you see database errors, ensure PostgreSQL is running and database exists.
echo Check console windows for debug output.
echo Press Ctrl+C in any window to stop that process.
echo.
pause
