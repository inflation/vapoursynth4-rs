#!/usr/bin/env python

from os.path import dirname, join
from pathlib import Path

from setuptools import Extension, setup

CURRENT_RELEASE = (
    Path("VAPOURSYNTH_VERSION")
    .read_text("utf8")
    .split(" ")[-1]
    .strip()
    .split("-")[0]
)

extra_data = {}

# Locate the vapoursynth dll inside the library directories first
# should we find it, it is a clear indicator that VapourSynth
# has been compiled by the user.
dll_path = join("msvc_project", "x64", "Debug", "VapourSynth.dll")

# Make sure the setup process copies the VapourSynth.dll into the site-package folder
print("Found VapourSynth.dll at:", dll_path)

extra_data["data_files"] = [(r"Lib\site-packages", [dll_path])]


setup(
    name="VapourSynth",
    description="A frameserver for the 21st century",
    url="https://www.vapoursynth.com/",
    download_url="https://github.com/vapoursynth/vapoursynth",
    author="Fredrik Mellbin",
    author_email="fredrik.mellbin@gmail.com",
    license="LGPL 2.1 or later",
    version=CURRENT_RELEASE,
    long_description="A modern replacement for Avisynth",
    platforms="All",
    ext_modules=[
        Extension(
            "vapoursynth",
            [join("src", "cython", "vapoursynth.pyx")],
            define_macros=[
                ("VS_USE_LATEST_API", None),
                ("VS_GRAPH_API", None),
                ("VS_CURRENT_RELEASE", CURRENT_RELEASE),
            ],
            libraries=["vapoursynth"],
            library_dirs=[dirname(dll_path)],
            include_dirs=[
                str(Path.cwd()),
                join("src", "cython"),
                join("src", "vsscript"),
            ],
        )
    ],
    setup_requires=[
        "setuptools>=18.0",
        "Cython",
    ],
    exclude_package_data={"": ("VAPOURSYNTH_VERSION",)},
    **extra_data,
)
