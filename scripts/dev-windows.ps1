$ErrorActionPreference = "Stop"
$tauriMode = if ($args.Count -gt 0 -and $args[0] -eq "build") { "build" } else { "dev" }

$cmakeCommand = Get-Command cmake.exe -ErrorAction SilentlyContinue
$cmakeBin = if ($cmakeCommand) { Split-Path -Parent $cmakeCommand.Source } elseif (Test-Path -LiteralPath "C:\Program Files\CMake\bin\cmake.exe") { "C:\Program Files\CMake\bin" } else { $null }

$llvmBin = if ($env:LIBCLANG_PATH -and (Test-Path -LiteralPath (Join-Path $env:LIBCLANG_PATH "libclang.dll"))) {
  $env:LIBCLANG_PATH
} elseif (Test-Path -LiteralPath "C:\Program Files\LLVM\bin\libclang.dll") {
  "C:\Program Files\LLVM\bin"
} else {
  $clangCommand = Get-Command clang.exe -ErrorAction SilentlyContinue
  if ($clangCommand -and (Test-Path -LiteralPath (Join-Path (Split-Path -Parent $clangCommand.Source) "libclang.dll"))) {
    Split-Path -Parent $clangCommand.Source
  } else { $null }
}

$vulkanSdk = if ($env:VULKAN_SDK -and (Test-Path -LiteralPath $env:VULKAN_SDK)) {
  $env:VULKAN_SDK
} elseif (Test-Path -LiteralPath "C:\VulkanSDK") {
  Get-ChildItem -LiteralPath "C:\VulkanSDK" -Directory | Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName
} else { $null }

$vswhere = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
$vsInstallation = if (Test-Path -LiteralPath $vswhere) {
  & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
} else { $null }
$msvcVars = if ($vsInstallation) {
  Join-Path $vsInstallation "VC\Auxiliary\Build\vcvars64.bat"
} else { $null }
$msvcToolsRoot = if ($vsInstallation) {
  Join-Path $vsInstallation "VC\Tools\MSVC"
} else { $null }
$msvcLinker = if ($msvcToolsRoot -and (Test-Path -LiteralPath $msvcToolsRoot)) {
  Get-ChildItem -LiteralPath $msvcToolsRoot -Directory | Sort-Object Name -Descending | ForEach-Object {
    $candidate = Join-Path $_.FullName "bin\Hostx64\x64"
    if (Test-Path -LiteralPath (Join-Path $candidate "link.exe")) { $candidate }
  } | Select-Object -First 1
} else { $null }

foreach ($requiredPath in @($cmakeBin, $llvmBin, $vulkanSdk, $msvcVars, $msvcLinker)) {
  if (!(Test-Path -LiteralPath $requiredPath)) {
    $hint = switch ($requiredPath) {
      $cmakeBin { "Install CMake: winget install Kitware.CMake" }
      $llvmBin { "Install LLVM: winget install LLVM.LLVM" }
      $vulkanSdk { "Install Vulkan SDK: winget install KhronosGroup.VulkanSDK" }
      $msvcVars { "Install Visual Studio Build Tools 2022 with the C++ workload and MSVC x64/x64 tools." }
      $msvcLinker { "Install Visual Studio Build Tools 2022 with the C++ workload and MSVC x64/x64 tools." }
      default { "Check the Windows development prerequisites in README.md." }
    }
    throw "Required Windows development component was not found. $hint"
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

  cmd.exe /d /s /c "call `"$msvcVars`" > nul && set `"LIBCLANG_PATH=$llvmBin`" && set `"VULKAN_SDK=$vulkanSdk`" && set `"CARGO_TARGET_DIR=$cargoTargetDir`" && set `"PATH=$msvcLinker;$cmakeBin;$llvmBin;$vulkanBin;$env:USERPROFILE\.cargo\bin;%PATH%`" && where link.exe && where glslc.exe && pnpm --filter @forge-wisper/desktop tauri $tauriMode"
if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}
