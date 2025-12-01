@echo off
echo Clearing all friend requests and contacts...
"C:\Program Files\PostgreSQL\17\bin\psql.exe" -U postgres -d xfmail -f clear_friends.sql
if %ERRORLEVEL% EQU 0 (
    echo.
    echo SUCCESS: All data cleared!
) else (
    echo.
    echo ERROR: Could not clear data
)
pause
