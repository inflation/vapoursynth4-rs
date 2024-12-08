$env:VSPYTHON_PATH = (Split-Path -Path (Get-Command python.exe).Path)
msbuild scripts/build-vs.slnf /t:Build /p:Configuration=Debug /p:Platform=x64 /m `
  /p:UseMultiToolTask=true /p:EnforceProcessCountAcrossBuilds=true `
  /p:DefineConstants=VS_USE_LATEST_API /p:DefineConstants=VSSCRIPT_USE_LATEST_API
