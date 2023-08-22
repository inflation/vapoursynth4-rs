import vapoursynth as vs
from vapoursynth import core

core.std.LoadPlugin(path=r'../target/debug/libsample_plugin.dylib')

clip = core.std.BlankClip(format=vs.YUV420P8, width=640, height=480, fpsnum=1, fpsden=1)
clip = core.filter.Filter(clip)

clip.set_output()
