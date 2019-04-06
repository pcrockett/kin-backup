Decrypt Executable Staging Area
===============================

As you probably know, Kin saves several `decrypt` executables with each backup package: one executable for each platform that Kin supports.

This is the "staging" area for all decrypt executables. All executables found in this directory will eventually be included in all backup packages.

So if you built a `kin_decrypt` package on a different platform, copy the resulting executable file here. Then during the build process, that executable will be packaged up with the `kin` executable, and `kin` will include it in any backup packages that it creates.

Also know that when you build the whole Kin project, the `decrypt` executable for your current platform will also automatically be copied here. So there will always be at least one `decrypt` executable in this directory during the build.
