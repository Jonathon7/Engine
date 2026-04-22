@echo off

echo Building Rust...
cargo build
if %errorlevel% neq 0 exit /b %errorlevel%

echo Building .NET...
dotnet build .\game\src -c Debug
if %errorlevel% neq 0 exit /b %errorlevel%

echo Copying outputs...
copy /y game\src\bin\Debug\net10.0\Game.dll target\debug\
copy /y game\src\bin\Debug\net10.0\Game.runtimeconfig.json target\debug\

echo Running...
target\debug\engine.exe
