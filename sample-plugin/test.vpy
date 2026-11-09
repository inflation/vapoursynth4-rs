import platform
from vapoursynth import GRAY8, core

if platform.system() == "Windows":
    path = r"../target/debug/sample_plugin.dll"
elif platform.system() == "Linux":
    path = r"../target/debug/libsample_plugin.so"
else:
    path = r"../target/debug/libsample_plugin.dylib"

core.std.LoadPlugin(path)

clip = core.std.BlankClip(width=16, height=16, format=GRAY8, color=[42])
clip = core.invert.Filter(clip, enabled=True)

clip.set_output()
