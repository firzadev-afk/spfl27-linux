Offside VAR 1.0-rc - VAR-style offside wall, camera sequence and banner for FL 2027 (Sider 7)
By Kris1982.

WHAT IT DOES
 - A white translucent wall stands on the offside line, and the offside zone is darker.
 - The replay freezes with the wall up and a cinematic camera plays: a lead-up of the play, the whole
   line seen square-on from behind the touchline, then a shot of the offside player (fly across
   the pitch round him, or one fixed angle).
 - A competition-style "OFFSIDE" banner sweeps in (the league logo, or a VAR badge).
 - Free camera: while it is frozen, press F or L3+R3 to fly the camera yourself; press again to go on.

REQUIREMENTS
 - Sider 7 with luajit.ext.enabled = 1, and nesalib.lua + CommonLib.lua (modules\lib, loaded before this
   module). They are not in this zip: they come with the other Kris1982 mods.
 - ReShade 6.x (the banner is a ReShade add-on + effect). Without it everything but the banner works.

INSTALL (everything goes in the game folder, next to FL_2027.exe)
 1. Unzip into the game folder (the one with FL_2027.exe), merging folders: the zip's root IS the game
    folder (SiderAddons\..., reshade-shaders\..., offside_banner.addon64 next to ReShade.ini).
 2. In SiderAddons\sider.ini make sure these lines are there:
        luajit.ext.enabled = 1
        cpk.root = ".\livecpk\root"
        lua.module = "offside.lua"        (after lib\nesalib.lua; once only)
 3. Start a match. In the ReShade effects list, enable "OffsideBanner" once if it is not on; the
    add-on turns it on by itself after that.
 To turn it off: put a ";" in front of the offside.lua line, and rename the folder
 livecpk\root\Asset\model\game2d\d2_offside_line (to remove the wall).

SETTINGS
Sider overlay, "offside" page: 9 / 0 choose a line, - / + change it, 8 default, 7 common/stadium.
 zone darkness, banner on / how long it stays, lead-up seconds and view, vertical view hold / distance /
 height, player shot (1 = fixed angle, 2 = fly across), end hold, free camera speed / invert left-right,
 debug freeze (holds the freeze N seconds - for testing; 0 = off).
The zip includes modules\offside.ini with the author's settings (back yours up when updating).
Saved to modules\offside.ini (more detail settings - angles, fly time - can be edited there; Shift+R reloads).
Free camera keys: I/K forward/back, J/L left/right, U/O down/up, arrows turn, N/M zoom, Shift fast.
Pad: left stick move, right stick turn, LT/RT down/up, RB/LB zoom.

KNOWN
 - First release candidate: tested with the Premier League and exhibition matches; report offsides where
   the camera stays on the vertical view or no wall shows (send the "offside:" lines of sider.log).
 - Do not load another module that declares ffi names starting "offside_" or "ofree_".
 - The wall model replaces Asset/model/game2d/d2_offside_line (fmdl) via livecpk.
