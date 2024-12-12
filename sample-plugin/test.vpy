import platform

from vapoursynth import core

if platform.system() == "Windows":
    path = "../target/debug/sample_plugin.dll"
elif platform.system() == "macOS":
    path = r"../target/debug/libsample_plugin.dylib"
else:
    path = r"../target/debug/libsample_plugin.so"

core.std.LoadPlugin(path)

clip = core.std.BlankClip()
clip = core.invert.Filter(clip, enabled=True)

clip.set_output()
