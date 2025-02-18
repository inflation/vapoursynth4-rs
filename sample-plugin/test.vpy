import platform

from vapoursynth import core

match platform.system():
    case "Windows":
        path=r"../target/debug/sample_plugin.dll"
    case "Linux":
        path=r"../target/debug/libsample_plugin.so"
    case "Darwin":
        path=r"../target/debug/libsample_plugin.dylib"

core.std.LoadPlugin(path)

clip = core.std.BlankClip()
clip = core.sample.Splice([clip, clip])

clip.set_output()
