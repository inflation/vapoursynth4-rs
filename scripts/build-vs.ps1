$env:VSPYTHON_PATH=(Split-Path -Path (Get-Command python.exe).Path)
cd vapoursynth\msvc_project
echo '
{
  "solution": {
    "path": "VapourSynth.sln",
    "projects": [
      "..\\zimg\\_msvc\\zimg\\zimg.vcxproj",
      "Core\\Core.vcxproj",
      "VSScript\\VSScript.vcxproj"
    ]
  }
}
' > build.slnf
msbuild build.slnf /t:Build /p:Configuration=Debug /p:Platform=x64 /m
