@echo off
rem   this is untested, if you use windows
rem   could you please change that, thank you.

rem   it might be worth it to use a ps1 script instead
setlocal
for /f "tokens=1,* delims= " %%a in ("%*") do set ARGV=%%b
set RUST_BACKTRACE=1 && cargo run -p %1 -- %ARGV%
endlocal
