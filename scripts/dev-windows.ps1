$ErrorActionPreference = "Stop"

$cmakeBin = "C:\Program Files\CMake\bin"
$llvmBin = "C:\Program Files\LLVM\bin"
$vulkanSdk = if ($env:VULKAN_SDK) { $env:VULKAN_SDK } elseif (Test-Path -LiteralPath "C:\VulkanSDK") {
  Get-ChildItem -LiteralPath "C:\VulkanSDK" -Directory | Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName
} else { $null }
$msvcVars = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
$msvcLinker = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64"

foreach ($requiredPath in @($cmakeBin, $llvmBin, $vulkanSdk, $msvcVars, $msvcLinker)) {
  if (!(Test-Path -LiteralPath $requiredPath)) {
    throw "Required Windows development component not found: $requiredPath"
  }
}

$vulkanBin = Join-Path $vulkanSdk "Bin"
$vulkanCompiler = Join-Path $vulkanBin "glslc.exe"
$cargoTargetDir = "C:\t"
if (!(Test-Path -LiteralPath $vulkanCompiler)) {
  throw "Vulkan shader compiler not found: $vulkanCompiler"
}

$env:LIBCLANG_PATH = $llvmBin
$env:VULKAN_SDK = $vulkanSdk
$env:CARGO_TARGET_DIR = $cargoTargetDir
$env:Path = "$msvcLinker;$cmakeBin;$llvmBin;$vulkanBin;$env:USERPROFILE\.cargo\bin;$env:Path"

cmd.exe /d /s /c "call `"$msvcVars`" > nul && set `"LIBCLANG_PATH=$llvmBin`" && set `"VULKAN_SDK=$vulkanSdk`" && set `"CARGO_TARGET_DIR=$cargoTargetDir`" && set `"PATH=$msvcLinker;$cmakeBin;$llvmBin;$vulkanBin;$env:USERPROFILE\.cargo\bin;%PATH%`" && where link.exe && where glslc.exe && pnpm --filter @forge-wisper/desktop tauri dev"
if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}
