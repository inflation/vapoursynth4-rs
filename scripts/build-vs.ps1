$VS_PORTABLE_PATH = "$PWD/vapoursynth-portable"
$VAPOURSYNTH_LIB_PATH = "$PWD/vapoursynth-portable/sdk/lib64"
$VS_INSTALL = "https://github.com/vapoursynth/vapoursynth/releases/download/R73/Install-Portable-VapourSynth-R73.ps1"

if (-not (Test-Path $VS_PORTABLE_PATH -PathType Container))
{`
    $script = Invoke-RestMethod $VS_INSTALL
  Invoke-Expression "& { $script } -Unattended"
}

Write-Output "$VS_PORTABLE_PATH;$VS_PORTABLE_PATH/Lib/site-packages" | Out-File -FilePath $env:GITHUB_PATH -Encoding utf8 -Append
Write-Output "VAPOURSYNTH_LIB_PATH=$VAPOURSYNTH_LIB_PATH" >> $env:GITHUB_ENV
