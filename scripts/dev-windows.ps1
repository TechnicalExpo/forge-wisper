$ErrorActionPreference = "Stop"

$cmakeBin = "C:\Program Files\CMake\bin"
$llvmBin = "C:\Program Files\LLVM\bin"
$msvcVars = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

foreach ($requiredPath in @($cmakeBin, $llvmBin, $msvcVars)) {
  if (!(Test-Path -LiteralPath $requiredPath)) {
    throw "Required Windows development component not found: $requiredPath"
  }
}

$env:LIBCLANG_PATH = $llvmBin
$env:Path = "$cmakeBin;$llvmBin;$env:USERPROFILE\.cargo\bin;$env:Path"

cmd.exe /d /s /c "call `"$msvcVars`" > nul && set `"LIBCLANG_PATH=$llvmBin`" && set `"PATH=$cmakeBin;$llvmBin;$env:USERPROFILE\.cargo\bin;%PATH%`" && pnpm --filter @forge-wisper/desktop tauri dev"
if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}
