# Simple XR2VR (Discontinued)
Was intended as a lightweight Linux OpenXR motion capture utility that outputs via the [VMC Protocol](https://protocol.vmc.info/english).

The reason for starting this project was that the [VMC-Client](https://github.com/sh-akira/VirtualMotionCapture) did not work under Linux, neither when running in a Wine Prefix or the Proton Prefix of a Game
(the software does work, but it finds no tracks/controllers/headset, so no tracking),
and while we could compile this project ourselfs, Unity does not support SteamVR target for Linux (or at least last I checked).  
Neither does [VNyan](https://suvidriel.itch.io/vnyan) work under the same conditions in the same way.

## Alternative
However, [Warudo](https://store.steampowered.com/app/2079120/Warudo/) does support SteamVR tracking and does work, and you can run it while running a VR game (you could have used VRChat already if you didn't want to game).  
Advantage is also that you don't need to run a seperate app for tracking and for rendering (although off loading the streaming to another machine is made less useful).  
I may make an addition to [KyloNeko's Linux Guide to VTubing](https://codeberg.org/KyloNeko/Linux-Guide-to-Vtubing) at some point to explain precisely how to set this up, 
but you can find some general guidance for setting up Warudo there.


## State
Project Discontinued due to other solution being found.  
  
In it's current state it contains the boilerplate to connect to openxr session and read the hand controller positions.
There was an attempt to read Vive Tracker positions, but this didn't work for some reason.  
Otherwise it imports already other crates for VMC and VRM, so if you didn't know these exist, you can check the `Cargo.toml`.  
  
But the actual difficult parts, the IK (Inverse Kinematics), is completly missing, so if you intend to implement anything like this project intended, you should probably start over.

## Building
Run:
```
make build
```
Depends on `openxr` library to build.  
You don't need a runtime to build, only to run.  
If you run the project without the runtime running then the runtime will be launched (this will clutter up std out).  
  
Requires rust 1.80 and higher
