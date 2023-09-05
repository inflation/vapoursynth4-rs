import vapoursynth as vs
from vapoursynth import core

core.std.LoadPlugin(path=r'../target/debug/libsample_plugin.dylib')

clip = core.std.BlankClip(format=vs.YUV420P8)
clip = core.invert.Filter(clip, enabled=True)

clip.set_output()
