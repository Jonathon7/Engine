@echo off

echo Building Rust (release)...
cargo build --release
if %errorlevel% neq 0 exit /b %errorlevel%

echo Building .NET (release)...
dotnet build .\game\src -c Release
if %errorlevel% neq 0 exit /b %errorlevel%

echo Copying outputs...
copy /y game\src\bin\Release\net10.0\Game.dll target\release\

echo Running...
target\release\engine.exe
