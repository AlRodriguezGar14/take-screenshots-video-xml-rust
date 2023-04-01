Known issues that need to be fixed:

- There is not a timecode to runtime connversion for the 25fps movies. This means, there will be always one frame of difference between the screenshot in the metadata and the one in the output. If this frame is critical, the python version of the code should work.

- There is not a differentiation for Drop Frame and Non Drop Frame 29,97 videos while doing the timecode to runtime conversion. We only work with one format, but It's something that I will think in the future about how to escalate it. 


TODO:

Test all the framerates available with multiple titles.
