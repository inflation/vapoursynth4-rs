$VAPOURSYNTH_LIB_PATH = "$PWD/vapoursynth/msvc_project/x64/Debug"
$DLL_PATH = "$VAPOURSYNTH_LIB_PATH/VapourSynth.dll"
$DLL_PATH2 = "$VAPOURSYNTH_LIB_PATH/VSScript.dll"

if (!((Test-Path -Path $DLL_PATH) -and (Test-Path -Path $DLL_PATH2))) {
  git clone --depth 1 --branch R70 https://github.com/vapoursynth/vapoursynth
  git -C vapoursynth clone --depth 1 --branch release-3.0.5 https://github.com/sekrit-twc/zimg

  py -3.12 -m pip install Cython setuptools

  $env:VSPYTHON_PATH = (py -3.12 -c "import sys; import os; print(os.path.dirname(sys.executable))")
  $env:VSPYTHON_PATH
  msbuild scripts/build-vs.slnf -m /t:"Clean;Build" /p:Configuration=Debug /p:Platform=x64 /m `
    /p:UseMultiToolTask=true /p:EnforceProcessCountAcrossBuilds=true `
    /p:DefineConstants=VS_USE_LATEST_API /p:DefineConstants=VSSCRIPT_USE_LATEST_API
}

Get-ChildItem -Path $VAPOURSYNTH_LIB_PATH -Filter '*.dll' | ForEach-Object {
  New-Item -ItemType SymbolicLink -Path (Join-Path 'target/debug/deps' $_.Name) -Target $_.FullName -Force
}

$env:PYTHONPATH = $env:VAPOURSYNTH_LIB_PATH
py -3.12 -c "from vapoursynth import core; print(str(core))"

"VAPOURSYNTH_LIB_PATH=${VAPOURSYNTH_LIB_PATH}" >> $env:GITHUB_ENV
"PYTHONPATH=${env:VAPOURSYNTH_LIB_PATH}" >> $env:GITHUB_ENV
