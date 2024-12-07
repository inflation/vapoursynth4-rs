from vapoursynth import core

core.std.LoadPlugin(path=r"../target/debug/sample_plugin.dll")

clip = core.std.BlankClip()
clip = core.invert.Filter(clip, enabled=True)

clip.set_output()
