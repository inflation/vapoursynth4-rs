$VAPOURSYNTH_LIB_PATH= "$((Get-ItemProperty HKCU:\SOFTWARE\VapourSynth -Name Path).Path)\sdk\lib64"

py -3.12 -c "from vapoursynth import core; print(str(core))"

"VAPOURSYNTH_LIB_PATH=${VAPOURSYNTH_LIB_PATH}" >> $env:GITHUB_ENV
