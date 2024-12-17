$VAPOURSYNTH_LIB_PATH = "$PWD/vapoursynth/msvc_project/x64/Debug"
$DLL_PATH = "$VAPOURSYNTH_LIB_PATH/VapourSynth.dll"
$DLL_PATH2 = "$VAPOURSYNTH_LIB_PATH/VSScript.dll"

if (!(Test-Path -Path $DLL_PATH)) {
  pip install Cython

  if (Test-Path -Path vapoursynth) { Remove-Item -Recurse -Force vapoursynth }
  git clone --depth 1 --branch R70 https://github.com/vapoursynth/vapoursynth
  git -C vapoursynth clone --depth 1 --branch release-3.0.5 https://github.com/sekrit-twc/zimg

  $env:VSPYTHON_PATH = (Split-Path -Path (Get-Command python.exe).Path)
  msbuild scripts/build-vs.slnf /t:Build /p:Configuration=Debug /p:Platform=x64 /m `
    /p:UseMultiToolTask=true /p:EnforceProcessCountAcrossBuilds=true `
    /p:DefineConstants=VS_USE_LATEST_API /p:DefineConstants=VSSCRIPT_USE_LATEST_API
}

New-Item -ItemType SymbolicLink -Target $DLL_PATH -Path target/debug/deps/VapourSynth.dll -Force
New-Item -ItemType SymbolicLink -Target $DLL_PATH2 -Path target/debug/deps/VSScript.dll -Force
Get-ChildItem $VAPOURSYNTH_LIB_PATH/VapourSynth.lib

$env:VAPOURSYNTH_LIB_PATH = $VAPOURSYNTH_LIB_PATH
"VAPOURSYNTH_LIB_PATH=${VAPOURSYNTH_LIB_PATH}" >> $env:GITHUB_ENV
