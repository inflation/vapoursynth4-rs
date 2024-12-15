import platform

from vapoursynth import core

match platform.system():
    case "Windows":
        core.std.LoadPlugin(path=r"../target/debug/sample_plugin.dll")
    case "Linux":
        core.std.LoadPlugin(path=r"../target/debug/libsample_plugin.so")
    case "Darwin":
        core.std.LoadPlugin(path=r"../target/debug/libsample_plugin.dylib")

clip = core.std.BlankClip()
clip = core.sample.Splice([clip, clip])

clip.set_output()
