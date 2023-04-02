Known issues that need to be fixed:

There is no timecode to runtime conversion for the 25fps movies. This means there will always be one frame of difference between the screenshot in the metadata and the one in the output. If this frame is critical, the python version of the code should work.

There is no differentiation for Drop Frame and Non-Drop Frame 29,97 videos while doing the timecode to runtime conversion. We only work with one format, but I will think about how to escalate it in the future.

TODO:

Test all the framerates available with multiple titles.



Python version (with different logic):
https://github.com/AlRodriguezGar14/take-screenshots-video-xml-python

This Rust version would have been impossible without the vtc-rs library:
https://github.com/opencinemac/vtc-rs
