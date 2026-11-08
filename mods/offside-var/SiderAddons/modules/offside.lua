-- SP Football Life offside graphics: darker offside zone + VAR camera on the offside freeze
-- (the white wall itself is the model in livecpk: Asset/model/game2d/d2_offside_line)

local m = { version = "6.5" }
local hex = memory.hex

-- The game colours the offside line model in code (FL_2027.exe ~0x1420cbadf):
--   mov dword [rbp-79], 0xff00beff   d2_offside_line_constant1: the line, yellow (RGBA bytes ff be 00 ff)
--   mov dword [rbp-75], 0x5f000000   d2_offside_line_constant2: the zone past the line, black, alpha 0x5f
-- The signature stops before the zone's alpha byte, so it still matches after a reload.
local SIGNATURE = "\xc7\x45\x87\xff\xbe\x00\xff\xc7\x45\x8b\x00\x00\x00"

-- Settings: in the Sider overlay (offside page) and in modules\offside.ini, per stadium if you like.
-- VAR camera while the game draws the offside wall (metres; the pitch is x -52.5..52.5, z -34..34),
-- whichever camera the game switches to underneath.
-- The camera shows the whole wall from its plane behind the main camera's touchline (the wall stands
-- exactly vertical in the picture), then moves to the offside player, a little off the wall's plane.
local function setting(ui, prop, def, step, lo, hi, fmt)
    return { ui = ui, prop = prop, def = def, fmt = fmt or "%0.1f",
             prevf = function(v) return math.max(lo, (tonumber(v) or def) - step) end,
             nextf = function(v) return math.min(hi, (tonumber(v) or def) + step) end }
end

-- a setting kept in modules\offside.ini but not shown in the overlay
local function hidden(s) s.hidden = true return s end

local SETTINGS = {
    setting("Offside zone darkness: %d  (0-255, game 95)", "zone_alpha", 160, 15, 0, 255, "%d"),
    setting("Offside banner at the vertical views: %d  (1 = on, 0 = off)", "banner", 1, 1, 0, 1, "%d"),
    setting("Offside banner: stays %0.1f s  (0 = while the view is vertical)", "banner_time", 8.0, 0.5, 0.0, 20.0),
    -- the sequence, in order; the players stay frozen with the wall from 2 to the end of 4
    setting("1. Lead-up: %0.2f s of play before the offside  (0 = off)", "leadup", 3.0, 0.25, 0.0, 10.0, "%0.2f"),
    setting("1. Lead-up view: picture %0.1f m  (bigger = wider)", "leadup_frame", 40.0, 5.0, 10.0, 120.0),
    setting("2. Vertical view: hold %0.2f s", "vertical_hold", 1.75, 0.25, 0.0, 20.0, "%0.2f"),
    setting("2. Vertical view: %0.1f m behind the touchline  (the whole line is always in the picture)", "vertical_back", 10.0, 1.0, 2.0, 40.0),
    setting("2. Vertical view: height %0.1f m", "vertical_height", 8.0, 0.5, 0.5, 40.0),
    -- after the vertical view, the shot of the offside player; angles round him: 0 = in the wall's
    -- plane on the main camera's touchline, + = toward the goal behind the line, - = the halfway side
    setting("3. Player shot: preset %d  (1 = to the player view, 2 = fly across the wall looking at him)", "player_preset", 2, 1, 1, 2, "%d"),
    -- the presets' details: in modules\offside.ini only, not in the overlay
    hidden(setting("3. To the offside player: %0.2f s", "player_move", 2.0, 0.25, 0.0, 10.0, "%0.2f")),
    hidden(setting("3. Preset 1 view: %d deg off the wall's plane", "player_angle", 15, 5, -60, 60, "%d")),
    -- (preset 2 crosses the pitch: from this touchline's side round him to the other touchline's side,
    -- 180 = in the wall's plane on the other side; - angles go round by the halfway side, + by the goal)
    hidden(setting("3. Preset 2 fly: from %d deg  (0 = this touchline)", "fly_from", 0, 15, -180, 180, "%d")),
    hidden(setting("3. Preset 2 fly: to %d deg  (-180 / 180 = the other touchline)", "fly_to", 165, 15, -180, 180, "%d")),
    hidden(setting("3. Preset 2 fly: %0.2f s", "fly_time", 4.0, 0.25, 0.5, 20.0, "%0.2f")),
    hidden(setting("3. Player view: %0.1f m from the player", "player_distance", 14.0, 1.0, 4.0, 40.0)),
    hidden(setting("3. Player view: height %0.1f m", "player_height", 5.0, 0.5, 0.5, 20.0)),
    hidden(setting("3. Player view: picture %0.1f m  (bigger = wider)", "player_frame", 8.0, 1.0, 2.0, 40.0)),
    setting("4. Player view: hold %0.2f s, then the replay goes on", "end_hold", 1.0, 0.25, 0.0, 10.0, "%0.2f"),
    -- debugging
    setting("Free camera: speed %0.1f m/s  (F or L3+R3 while frozen: fly it yourself; again: back to the game)", "free_speed", 10.0, 1.0, 1.0, 40.0),
    setting("Free camera: invert left/right: %d  (1 = on; moving and turning, sticks and keys)", "free_invert", 1, 1, 0, 1, "%d"),
    setting("Debug: freeze at least %d s  (0 = off; the wall and the players held this long)", "debug_freeze", 0, 10, 0, 300, "%d"),
}
-- zone_alpha goes straight to the game's colour code (base "zone"); the rest are read each frame
local SETTINGS_INFO = {}
for _, s in ipairs(SETTINGS) do
    SETTINGS_INFO[s.prop] = { base = s.prop == "zone_alpha" and "zone" or "none", offs = 0,
                              format = s.prop == "zone_alpha" and "u8" or "f", len = 4, def = s.def,
                              save_as = s.fmt }
end
SETTINGS_INFO.zone_alpha.len = 1
local OVERLAY = {}            -- the overlay's rows (the helper's restore key resets only these)
for _, s in ipairs(SETTINGS) do
    if not s.hidden then OVERLAY[#OVERLAY + 1] = s end
end

local helper                  -- nesalib overlay/ini helper (m.init)
local DEFAULTS = {}
for _, s in ipairs(SETTINGS) do DEFAULTS[s.prop] = s.def end

-- the current settings as numbers (the overlay's, or the defaults before the helper exists)
local function cfg()
    local s = helper and helper.get_settings() or DEFAULTS
    local out = {}
    for k, d in pairs(DEFAULTS) do out[k] = tonumber(s[k]) or d end
    return out
end

local VAR_RAMP = 0.5          -- seconds to move from the game camera to ours
local EYE_X_MAX = 54.0        -- the eye stays this far from the centre along the pitch (goal lines: 52.5)

-- Longer freeze: the replay freeze lasts as long as the wall's 60-frame animation (18 frames to grow,
-- full size until 42, shrink to 60). The wall is held at full size through the vertical view, the move
-- and the end view (steps 2-4 of the settings); then it shrinks and the replay goes on.
local WALL_FULL = 30          -- animation frame held (18..42 = full size)
local WALL_FRAMES = 60        -- the animation's last frame (the model is hidden there)

-- The replay player advances in FL_2027.exe 0x1421738e0(player, speed in xmm1), called by every replay
-- step (14 callers, each with its own speed; it has its own path for speed 0). While the wall is held,
-- a hook zeroes the speed, so the players stay frozen with the wall; during the lead-up it plays at a
-- set speed. Its data: +0 mode byte (0 = the game's speed, 1 = 0, 2 = the float at +4), +8 the player
-- it was last called with.
-- Its first 20 bytes (sub rsp,38 / cmp dword [rcx],5 / movaps xmm5,xmm1 / mov r8,rcx /
-- mov dword [rcx+18],0) move to the cave; the flags of that cmp survive the jump back.
local SPEED_ENTRY = "\x48\x83\xec\x38\x83\x39\x05\x0f\x28\xe9\x4c\x8b\xc1\xc7\x41\x18\x00\x00\x00\x00"
-- right after it (jne / movaps [rsp],xmm8): finds the patched entry on a reload
local SPEED_AFTER = "\x0f\x85\x68\x01\x00\x00\x44\x0f\x29\x04\x24"
local SPEED_DATA = 0x60       -- the cave's data, after its code (before 3.5: a flag at +0x40)

-- Lead-up: the game's offside replay shows the clip from the offside on. The replay player is a ring
-- buffer of recorded frames (660 = 11 s): +0x14 position (float), +0x23a ring length, +0x23c clip start,
-- +0x240 clip end (u16 ring indices). The clip starts at the last restart of play (frames before it are
-- stale: players standing at the restart), so the position moves back inside the clip; the replay plays
-- at normal speed with the line model hidden (animation frame 60: the update hides it, and shows it again
-- below 60), and the offside begins when the position is back where it was. The game draws one frame of
-- the replay before Lua can rewind it, so the lead-up cuts to our own wide camera (a cut, as on TV,
-- instead of the players jumping back in the same picture), which then glides into the vertical view.
local LEADUP_FPS = 54         -- recorded frames a second (one a game frame; measured 180 in 3.34 s)
local LEADUP_AHEAD = 8.0      -- the wide view centres this far to the halfway side of the line (m)
local LEADUP_HEIGHT = 18.0    -- its eye: height, and distance behind the main camera's touchline
local LEADUP_BACK = 22.0
local LEADUP_GLIDE = 1.0      -- seconds to glide from the wide view into the vertical view

-- Offside line model update (FL_2027.exe 0x1420cb980, this = rcx): it shows the model when it sets
-- byte [this+0x27d] = 1 and hides it when it clears it. The hook saves `this` so Lua can read the flag.
-- Found by the code 0x2a bytes in (mov rdi,rcx / mov rsi,[rax+388] / mov r14,[rax+270]);
-- its first 14 bytes (mov [rsp+18],rsi / push rbp / push rdi / push r14 / lea rbp,[rsp-47]) move to the cave.
local MODEL_ANCHOR = "\x48\x8b\xf9\x48\x8b\xb0\x88\x03\x00\x00\x4c\x8b\xb0\x70\x02\x00\x00"
local MODEL_ENTRY = "\x48\x89\x74\x24\x18\x55\x57\x41\x56\x48\x8d\x6c\x24\xb9"

-- Camera system update (FL_2027.exe ~0x141eaf8a3): every frame it calls the active camera's
-- update (vtable +0x20) for all match cameras (play, offside, ...), with one output struct:
--   mov rcx,[rbx] / mov [rbx+8],rcx / mov rax,[rcx] / lea rdx,[out] / call [rax+20] / movzx edx,byte [flag]
-- output: +0 look-at xyz, +0xc eye xyz, +0x30 vertical fov (radians)
local CAM_PREFIX = "\x48\x8b\x0b\x48\x89\x4b\x08\x48\x8b\x01"
local CAM_ORIG = "\x48\x8d\x15"   -- + disp32, then call [rax+20] and movzx edx,byte [rip+disp32]
-- CameraOffside constructor, right before it loads its vtable (lea rax,[vtable])
local OFFSIDE_CTOR = "\x88\x41\x44\xc7\x41\x48\x00\x00\x00\x06\xc7\x41\x4c\x00\x00\x80\x3f" ..
    "\xc7\x41\x50\xcd\xcc\x4c\x3f\xc7\x41\x54\x00\x00\x80\x3f\x48\x8d\x05"
-- match manager global (same code BroadCastCam uses); out-of-play info [global+0x270], line x at +0x15a8
local MATCH_GLOBAL = "\x83\x79\x2c\x05\x4c\x8b\xfa\x48\x8b\x05"

local function jmp_abs(target)  -- jmp qword ptr [rip+0]; dq target
    return "\xff\x25\x00\x00\x00\x00" .. memory.pack("u64", target)
end

-- cave data: +0 weight, +4 look-at target xyz, +0x10 frame counter, +0x14 game look-at xyz,
--            +0x20 game eye xyz, +0x34 game fov, +0x3c eye target xyz, +0x48 fov target,
--            +0x58 vtable of the active camera, +0x60 up target xyz, +0x6c near clip target
-- The up vector is blended too: the game tilts it with its own camera's pitch, which turns into a
-- roll when our camera looks another way. So is the near clip distance (output +0x34, handed to the
-- renderer right after): the game's far-off replay cameras clip everything nearer than several metres,
-- which cut the ground away in front of our closer camera.
local NEAR_CLIP = 0.5
local CAM_CODE_MAX, CAM_DATA_SIZE = 0x200, 0x70   -- the data follows the code (versions before 3.2: at +0x180)
local function build_cam_cave(out, flag, data, ret)
    local blend = {}
    for _, f in ipairs({ { 0x00, 0x04 }, { 0x04, 0x08 }, { 0x08, 0x0c },     -- look-at
                         { 0x0c, 0x3c }, { 0x10, 0x40 }, { 0x14, 0x44 },     -- eye
                         { 0x18, 0x60 }, { 0x1c, 0x64 }, { 0x20, 0x68 },     -- up
                         { 0x30, 0x48 }, { 0x34, 0x6c } }) do                -- fov, near clip
        blend[#blend + 1] = "\xf3\x0f\x10\x49" .. string.char(f[1]) ..   -- movss xmm1,[rcx+k]  game value
            "\xf3\x0f\x10\x50" .. string.char(f[2]) ..                   -- movss xmm2,[rax+t]  target
            "\xf3\x0f\x5c\xd1" ..                                        -- subss xmm2,xmm1
            "\xf3\x0f\x59\xd0" ..                                        -- mulss xmm2,xmm0
            "\xf3\x0f\x58\xca" ..                                        -- addss xmm1,xmm2
            "\xf3\x0f\x11\x49" .. string.char(f[1])                      -- movss [rcx+k],xmm1
    end
    blend = table.concat(blend)
    return "\x48\xba" .. memory.pack("u64", out) ..   -- mov rdx,out           (replaced lea)
        "\xff\x50\x20" ..                              -- call [rax+20]         camera update (replaced)
        "\x48\xb8" .. memory.pack("u64", data) ..      -- mov rax,data
        "\xff\x40\x10" ..                              -- inc dword [rax+10]
        "\x48\xb9" .. memory.pack("u64", out) ..       -- mov rcx,out
        "\x0f\x10\x09" ..                              -- movups xmm1,[rcx]     game look-at, eye.x
        "\x0f\x11\x48\x14" ..                          -- movups [rax+14],xmm1
        "\x0f\x10\x49\x10" ..                          -- movups xmm1,[rcx+10]  eye.y, eye.z
        "\x0f\x11\x48\x24" ..                          -- movups [rax+24],xmm1
        "\xf3\x0f\x10\x49\x30" ..                      -- movss xmm1,[rcx+30]   fov
        "\xf3\x0f\x11\x48\x34" ..                      -- movss [rax+34],xmm1
        "\x48\x8b\x13" ..                              -- mov rdx,[rbx]         active camera
        "\x48\x89\x50\x50" ..                          -- mov [rax+50],rdx
        "\x48\x8b\x12" ..                              -- mov rdx,[rdx]         its vtable
        "\x48\x89\x50\x58" ..                          -- mov [rax+58],rdx
        "\xf3\x0f\x10\x40\x00" ..                      -- movss xmm0,[rax]      weight
        "\x0f\x57\xc9" ..                              -- xorps xmm1,xmm1
        "\x0f\x2f\xc1" ..                              -- comiss xmm0,xmm1
        "\x0f\x86" .. memory.pack("i32", #blend) ..    -- jbe done
        blend ..
        -- done:
        "\x48\xb8" .. memory.pack("u64", flag) ..      -- mov rax,flag
        "\x0f\xb6\x10" ..                              -- movzx edx,byte [rax]  (replaced)
        jmp_abs(ret)
end

local cam_data, cam_out, offside_vt, match_global
local model_data          -- +0 the line model update's `this`, +8 call counter
local last_vt, ev, last_w = nil, nil, 0
local model_calls, model_seen = nil, 0
local speed_data          -- the replay player hook's data (SPEED_DATA)
local speed_data_v1       -- an older hook in place: freeze only, no lead-up
local replay_mode = 0

-- Sider hands back addresses as pointer cdata (memory.search_process, allocate_codecave), which never ==
-- a Lua number and which tonumber() turns into nil: compare plain numbers (addresses fit in a double)
local function addr(p)
    if type(p) == "cdata" then return tonumber(ffi.cast("uint64_t", p)) end
    return tonumber(p)
end
local function u64at(a) return addr(memory.unpack("u64", memory.read(a, 8))) end
local function fat(a) return memory.unpack("f", memory.read(a, 4)) end
local function valid_ptr(p) return p and p > 0x10000 and p < 0x7fffffffffff end
local function sign(v) return v < 0 and -1 or 1 end

local function smooth(t) t = math.max(0, math.min(1, t)) return t * t * (3 - 2 * t) end
-- The vertical view: the eye in the wall's plane (x = the line), a fixed distance behind the main
-- camera's touchline. It is aimed and zoomed from the wall's four corners (both touchlines, the ground
-- and the top), so the whole line is always in the picture, wherever the line and the game camera are.
local WALL_HEIGHT = 3.0       -- the wall model stands 0..3 m (content/offside-wall/wall.py)
local VERTICAL_MARGIN = 1.15  -- the picture is this much taller than the wall's corners
local function vertical_camera(x, side, c)
    local back, h = math.max(2, c.vertical_back), c.vertical_height
    local lo, hi = math.huge, -math.huge
    for _, d in ipairs({ back, back + 68 }) do         -- the near and the far touchline
        for _, y in ipairs({ 0, WALL_HEIGHT }) do
            local e = math.atan((y - h) / d)           -- elevation from the eye
            lo, hi = math.min(lo, e), math.max(hi, e)
        end
    end
    local d = 34 + back                                -- aim at the middle of the pitch
    return { x, h + math.tan((lo + hi) / 2) * d, 0, x, h, side * d, math.min(1.2, (hi - lo) * VERTICAL_MARGIN) }
end

-- the offside line x as the line model update saw it on its last call (0 = not captured).
-- (Read in the hook: from Lua the out-of-play info pointer reads as null at that moment.)
local function line_hooked()
    if not model_data then return nil end
    local x = fat(model_data + 0xc)
    if x ~= 0 and x == x and math.abs(x) <= 60 then return x end
    return nil
end

-- The wall is drawn while the line model's flag [this+0x27d] is 1 and its animation frame
-- [this+0x280] is below 60: frames 0-18 grow, 18-42 full size, 42-60 shrink, then the model is hidden
-- (the flag stays 1). Returns the object and the frame when drawn (hidden counts too during the
-- lead-up, which hides it). Only read while the game still runs its update (the object is alive): the
-- update must have been called within the last quarter second.
local function wall_drawn(now, leading)
    if not model_data then return nil end
    local calls = memory.unpack("u32", memory.read(model_data + 8, 4))
    if calls ~= model_calls then
        model_calls, model_seen = calls, now
    elseif now - model_seen > 0.25 then
        return nil
    end
    local obj = u64at(model_data)
    if not valid_ptr(obj) or memory.unpack("u8", memory.read(obj + 0x27d, 1)) ~= 1 then return nil end
    local n = memory.unpack("u32", memory.read(obj + 0x280, 4))
    if n >= WALL_FRAMES and not leading then return nil end
    return obj, n
end

-- Memory check (VirtualQuery). Own FFI names: the FFI namespace is shared by all modules, and sharing
-- BroadCastCam's (bcc_VirtualQuery) broke when its newer version declared it with another struct type
-- ("cannot convert 'struct ..' to 'struct .. *'" every frame).
local mbi, MBI_SIZE
if ffi then
    if not ffi.__offside_vq_cdef then
        ffi.cdef[[
        typedef struct {
            void* BaseAddress; void* AllocationBase; uint32_t AllocationProtect; uint16_t PartitionId;
            size_t RegionSize; uint32_t State; uint32_t Protect; uint32_t Type;
        } offside_MBI;
        size_t offside_VirtualQuery(const void* p, offside_MBI* mbi, size_t len) __asm__("VirtualQuery");
        ]]
        ffi.__offside_vq_cdef = true
    end
    mbi = ffi.new("offside_MBI")
    MBI_SIZE = ffi.sizeof(mbi)
end

local function readable(p, n)
    if not mbi or not valid_ptr(p) then return false end
    if ffi.C.offside_VirtualQuery(ffi.cast("void*", p), mbi, MBI_SIZE) == 0 then return false end
    if mbi.State ~= 0x1000 or bit.band(mbi.Protect, 0x101) ~= 0 or bit.band(mbi.Protect, 0xEE) == 0 then
        return false
    end
    return p + n <= addr(mbi.BaseAddress) + tonumber(mbi.RegionSize)
end

-- the replay speed: 0 = the game's, 1 = frozen, 2 = `speed` (frames a call)
local function set_replay(mode, speed)
    if speed_data and mode ~= replay_mode then
        if speed then memory.write(speed_data + 4, memory.pack("f", speed)) end
        memory.write(speed_data, string.char(mode))
        replay_mode = mode
    end
end
local function freeze_replay(on) set_replay(on and 1 or 0) end

local function u16at(a) return memory.unpack("u16", memory.read(a, 2)) end

-- move the position back inside the clip for the lead-up (see LEADUP_FPS); ev.lead while it plays
local function start_leadup(ev, seconds)
    if seconds <= 0 or not speed_data or speed_data_v1 then return end
    local p = u64at(speed_data + 8)
    if not readable(p, 0x244) or memory.unpack("u32", memory.read(p, 4)) ~= 5 then return end
    local len, a, b = u16at(p + 0x23a), u16at(p + 0x23c), u16at(p + 0x240)
    local pos = fat(p + 0x14)
    if len == 0 or a >= len or b >= len or not (pos >= 0 and pos < len) then return end
    local k = math.min(math.floor(seconds * LEADUP_FPS), math.floor((pos - a) % len))
    if k < LEADUP_FPS / 2 then return end   -- the clip starts right before
    memory.write(p + 0x14, memory.pack("f", (pos - k) % len))
    set_replay(2, 1.0)
    ev.lead = { p = p, pos = pos, len = len, k = k, t0 = ev.t0 }
end

-- while the lead-up plays: true (the line model hidden). Back at the old position (or after a time
-- out): the replay freezes, the wall grows from its first frame and the event clock starts again.
local function leadup(ev, obj, now, seconds)
    local ld = ev.lead
    if not ld then return false end
    local left = (ld.pos - fat(ld.p + 0x14)) % ld.len   -- frames to go (wraps to ~len once past)
    if left > 0.5 and left <= ld.k + 1 and now - ld.t0 < seconds + 3 then
        if obj then memory.write(obj + 0x280, memory.pack("u32", WALL_FRAMES)) end
        return true
    end
    ev.lead = nil
    if obj then
        -- the wall is up already: it grows now, the players frozen, and the sequence starts
        set_replay(1)
        memory.write(obj + 0x280, memory.pack("u32", 0))
        ev.t0, ev.prewall = now, nil
    else
        -- before the wall (started with the replay): the game's own slow run-in plays on to the wall
        set_replay(0)
    end
    return false
end

-- keep the wall at full size: once it has grown, put its animation frame back each frame until the
-- player view is done (the replay freeze is timed with the wall's animation)
local function hold_wall(ev, obj, n, t, hold)
    if t < hold and n >= WALL_FULL then
        ev.held = true
        memory.write(obj + 0x280, memory.pack("u32", WALL_FULL))
        freeze_replay(true)
    elseif ev.held and not ev.released then
        ev.released = true
        freeze_replay(false)
    end
end

-- Offside banner: offside_banner.addon64 (a ReShade add-on in the game folder) exports the state that
-- OffsideBanner.fx draws from; it is found by name in this process (looked up every 2 s until found).
-- The tournament id picks the competition's logo and colours (offside_logo_map.png).
local BANNER_FADE = 0.3       -- seconds to fade out when the camera leaves the vertical view
local banner, banner_look_t = nil, -10
if ffi and not ffi.__offside_banner_cdef2 then
    ffi.cdef[[
    typedef struct { float time; float alpha; uint32_t seq; uint32_t tournament; } offside_banner_state2;
    void* ob_GetModuleHandleA(const char* name) __asm__("GetModuleHandleA");
    void* ob_GetProcAddress(void* module, const char* name) __asm__("GetProcAddress");
    ]]
    ffi.__offside_banner_cdef2 = true
end

local function show_banner(now, time, alpha, tournament)
    if not banner and ffi and now - banner_look_t >= 2 then
        banner_look_t = now
        local h = ffi.C.ob_GetModuleHandleA("offside_banner.addon64")
        local p = h ~= nil and ffi.C.ob_GetProcAddress(h, "offside_banner_state")
        if p and p ~= nil then
            banner = ffi.cast("offside_banner_state2*", p)
            log("offside: banner add-on found")
        end
    end
    if banner and (time >= 0 or banner.time >= 0) then
        banner.time, banner.alpha, banner.seq = time, alpha, banner.seq + 1
        if tournament then banner.tournament = tournament end
    end
end

-- the competition for the banner's logo: the tournament, or in an exhibition match (65535) the league
-- both teams play in (CommonLib, as the scoreboards pick theirs); otherwise none (the VAR badge)
local function banner_tournament(ctx)
    local tid = tonumber(ctx.tournament_id) or 0
    local lib = ctx.common_lib
    if tid == 65535 and lib and lib.tid_same_league then
        local same = lib.tid_same_league(ctx.home_team, ctx.away_team)
        return tonumber(same) or tid
    end
    return tid
end

-- at a vertical view (eye within about 10 degrees of the wall's plane, the camera blended in) the banner
-- sweeps in and stays `life` seconds (0: while the view stays vertical), then fades out; with a set
-- life it shows once an offside
local function update_banner(ev, now, on, a, life)
    local vertical = on and math.cos(a) ^ 2 > 0.97
    if not ev.banner_t0 and vertical and not ev.banner_done then
        ev.banner_t0 = now
    end
    local t = ev.banner_t0 and now - ev.banner_t0
    if t and on and (life > 0 and t < life or life <= 0 and vertical) then
        ev.banner_off = nil
        show_banner(now, t, 1, ev.tournament)
    elseif ev.banner_t0 then
        ev.banner_done = life > 0
        ev.banner_off = ev.banner_off or now
        local fade = 1 - (now - ev.banner_off) / BANNER_FADE
        if fade > 0 then
            show_banner(now, now - ev.banner_t0, fade)
        else
            ev.banner_t0, ev.banner_off = nil, nil
            show_banner(now, -1, 0)
        end
    end
end

local function set_weight(w)
    if w ~= 0 or last_w ~= 0 then
        memory.write(cam_data, memory.pack("f", w))
        last_w = w
    end
end

-- our camera's target: { look x, y, z, eye x, y, z, vertical fov }
local function aim(cam)
    local f = function(v) return memory.pack("f", v) end
    memory.write(cam_data + 0x04, f(cam[1]) .. f(cam[2]) .. f(cam[3]))
    memory.write(cam_data + 0x3c, f(cam[4]) .. f(cam[5]) .. f(cam[6]))
    memory.write(cam_data + 0x48, f(cam[7]))
end

-- the lead-up's camera: wide, high behind the main camera's touchline, centred a little to the halfway
-- side of the line, where the play comes from
local function leadup_camera(ev, picture)
    local cx = ev.x - sign(ev.x) * LEADUP_AHEAD
    local look_z, eye_y, eye_z = ev.z * 0.5, LEADUP_HEIGHT, ev.side * (34 + LEADUP_BACK)
    local d = math.sqrt(eye_y ^ 2 + (eye_z - look_z) ^ 2)
    return { cx, 0, look_z, cx, eye_y, eye_z, math.max(0.1, math.min(1.2, 2 * math.atan(picture / 2 / d))) }
end

-- The replay player's start: the lead-up starts with the offside camera only right after it. The hook
-- saves whatever its function was last called with, not always the player: the tracked player stays
-- until it stops playing (state 5).
local replay = {}         -- { p = the replay player, t_start = when it started }
local leadup_replay       -- the replay (its start time) whose offside camera started a lead-up
-- the last replay player seen playing, kept after it stops: its replayed humans stay in it (in game, a
-- replay near the goal stopped being "playing" while frozen, and the offside player was lost)
local last_replay_p
-- A replay player is told by its replayed humans too: [p+0x128] holds a match::ReplayHuman (vtable
-- 0x142bf7d58, set by its constructor 0x14217d340). In game the hook also saw other objects whose
-- first dword happened to be 5: tracked as the player, the replay start was missed (no lead-up) and
-- the offside player was not found.
local REPLAY_HUMAN_VT = 0x142bf7d58
local function track_replay(now)
    local function playing(p)
        if not (p and readable(p, 0x244) and memory.unpack("u32", memory.read(p, 4)) == 5) then return false end
        local h = u64at(p + 0x128)
        return valid_ptr(h) and readable(h, 8) and u64at(h) == REPLAY_HUMAN_VT
    end
    local p = speed_data and not speed_data_v1 and u64at(speed_data + 8)
    if not playing(p) then p = replay.p end
    if not playing(p) then
        replay = {}
    else
        last_replay_p = p
        if p ~= replay.p then replay = { p = p, t_start = now } end
    end
end

-- The offside player in the replay (debug pan). Which player: the out-of-play info's index
-- [[g]+0x270]+0x1590 (0-10 home, 11-21 away; MatchFocus reads it as the fouler; from Lua [g+0x270]
-- reads as null on some frames, so it is retried). Where: the replay player creates its 26 replayed
-- humans at [player + 0x128 + i*8] (FL_2027.exe 0x142173cb0) with that same numbering as their id
-- (+0x28; team by 0x141e534f0: 0-10, 11-21, then the referees) and the replayed position at +0x1d8
-- (slide-turf.lua uses it). Frozen with the replay, so it is the pose at the offside.
local RH_LIST, RH_ID, RH_POS = 0x128, 0x28, 0x1d8

local function offside_index()
    local g = match_global and u64at(match_global)
    if not valid_ptr(g) then return nil end
    local mi = u64at(g + 0x270)
    if not valid_ptr(mi) then return nil end
    local i = memory.unpack("u32", memory.read(mi + 0x1590, 4))
    if i < 22 then return i end
end

-- replayed human i: its position, or nil and why not
local function human_pos(i)
    local p = replay.p or last_replay_p
    if not p then return nil, "no replay player seen yet" end
    local h = u64at(p + RH_LIST + i * 8)
    if not readable(h, RH_POS + 12) then
        return nil, string.format("no replayed human %d in replay player %s", i, hex(p))
    end
    local id = memory.unpack("u32", memory.read(h + RH_ID, 4))
    if id ~= i then return nil, string.format("replayed human %d has id %d", i, id) end
    local x, y, z = fat(h + RH_POS), fat(h + RH_POS + 4), fat(h + RH_POS + 8)
    if not (x == x and z == z and math.abs(x) < 60 and math.abs(z) < 45 and math.abs(y) < 5) then
        return nil, string.format("replayed human %d off the pitch (%.1f, %.1f, %.1f)", i, x, y, z)
    end
    return { x, y, z }
end

-- once the replay is frozen: find the offside player (retried each frame; each failure logged once)
local function find_player(ev)
    if ev.player then return ev.player end
    local i = ev.player_i or offside_index()
    local pos, why = nil, "offside player index not read yet"
    if i then
        ev.player_i = i
        pos, why = human_pos(i)
    end
    if not pos then
        if why ~= ev.player_why then
            ev.player_why = why
            log("offside: offside player: " .. why)
        end
        return nil
    end
    ev.player = pos
    local past = (pos[1] - ev.x) * sign(ev.x)   -- metres past the line toward its goal
    log(string.format("offside: offside player #%d (%s %d) at (%.2f, %.2f, %.2f), line x %.2f, %.2f m past the line%s",
        i, i < 11 and "home" or "away", i % 11, pos[1], pos[2], pos[3], ev.x, past,
        past > -1 and "" or " - BEHIND the line: wrong player?"))
    if not replay.p then   -- read from the kept player: log its state (5 = playing)
        local p = last_replay_p
        log(string.format("offside: (from the last replay player %s, state %d)", hex(p),
            readable(p, 4) and memory.unpack("u32", memory.read(p, 4)) or -1))
    end
    local all = {}
    for k = 0, 21 do
        local q = human_pos(k)
        all[#all + 1] = q and string.format("%d:(%.1f,%.1f)", k, q[1], q[3]) or (k .. ":-")
    end
    log("offside: replayed players " .. table.concat(all, " "))
    return pos
end

-- The player view: looking at the offside player from near the wall's plane on the main camera's
-- touchline, turned player_angle round him (+ toward the goal behind the line): the wall stands
-- slightly off vertical in the picture, the player just past it (at 0 it stands exactly vertical).
local function player_camera(ev, c, pos, deg)
    local a = math.rad(deg)
    local eye_x = ev.x + sign(ev.x) * c.player_distance * math.sin(a)
    eye_x = math.max(-EYE_X_MAX, math.min(EYE_X_MAX, eye_x))
    local eye_z = math.max(-44, math.min(44, pos[3] + ev.side * c.player_distance * math.cos(a)))
    local d = math.sqrt((eye_x - pos[1]) ^ 2 + (c.player_height - 1) ^ 2 + (eye_z - pos[3]) ^ 2)
    return { pos[1], 1.0, pos[3], eye_x, c.player_height, eye_z,
             math.max(0.05, math.min(1.2, 2 * math.atan(c.player_frame / 2 / d))) }
end

-- Free camera: while the replay is frozen, F (keyboard) or L3+R3 (gamepad) hands the camera over and
-- keeps everything frozen until pressed again; then the replay goes on. Read straight from the
-- keyboard (GetAsyncKeyState) and the pad (XInput), only while the game window is in front: Sider
-- sends its key and pad events only to the module whose overlay is open.
--   keyboard: I/K forward/back, J/L left/right, U/O down/up, arrows turn, N/M zoom in/out, Shift fast
--   gamepad:  left stick move, right stick turn, LT/RT down/up, RB/LB zoom in/out
local FREE_KEY = 0x46                     -- F
local FREE_PAD = 0x0040 + 0x0080          -- L3 + R3, together
local FREE_KEYS = { fwd = 0x49, back = 0x4b, left = 0x4a, right = 0x4c, down = 0x55, up = 0x4f,
                    zin = 0x4e, zout = 0x4d, tl = 0x25, tu = 0x26, tr = 0x27, td = 0x28, fast = 0x10 }
local FREE_FAST, FREE_TURN, FREE_ZOOM = 3.0, 1.6, 0.8   -- x speed with Shift; rad/s; zoom rate
local PAD_DEAD = 0.24                     -- stick dead zone

if ffi and not ffi.__offside_free_cdef then
    ffi.cdef[[
    typedef struct { uint16_t buttons; uint8_t lt, rt; int16_t lx, ly, rx, ry; } ofree_PAD;
    typedef struct { uint32_t packet; ofree_PAD pad; } ofree_STATE;
    typedef uint32_t (*ofree_XInputGetState)(uint32_t user, ofree_STATE* state);
    void* ofree_LoadLibraryA(const char* name) __asm__("LoadLibraryA");
    short ofree_GetAsyncKeyState(int key) __asm__("GetAsyncKeyState");
    void* ofree_GetForegroundWindow() __asm__("GetForegroundWindow");
    uint32_t ofree_GetWindowThreadProcessId(void* window, uint32_t* pid) __asm__("GetWindowThreadProcessId");
    uint32_t ofree_GetCurrentProcessId() __asm__("GetCurrentProcessId");
    ]]
    ffi.__offside_free_cdef = true
end

local xinput, pad_state, pid_out   -- XInputGetState (false: none), its buffers
local pad_user, pad_scan_t = nil, -10
local function get_pad()
    if xinput == nil then
        xinput = false
        pad_state, pid_out = ffi.new("ofree_STATE"), ffi.new("uint32_t[1]")
        for i, name in ipairs({ "xinput1_4.dll", "xinput1_3.dll", "xinput9_1_0.dll", "xinput1_4.dll" }) do
            -- the one the game loaded; else load xinput1_4 (in Windows since 8)
            local h = i < 4 and ffi.C.ob_GetModuleHandleA(name) or ffi.C.ofree_LoadLibraryA(name)
            local f = h ~= nil and ffi.C.ob_GetProcAddress(h, "XInputGetState")
            if f and f ~= nil then
                xinput = ffi.cast("ofree_XInputGetState", f)
                log("offside: free camera: gamepad through " .. name)
                break
            end
        end
        if not xinput then log("offside: free camera: no XInput, keyboard only") end
    end
    if not xinput then return nil end
    -- the connected pad; the others (slow to ask when nothing is plugged in) once a second
    if pad_user and xinput(pad_user, pad_state) == 0 then return pad_state.pad end
    pad_user = nil
    local now = os.clock()
    if now - pad_scan_t < 1 then return nil end
    pad_scan_t = now
    for user = 0, 3 do
        if xinput(user, pad_state) == 0 then
            pad_user = user
            return pad_state.pad
        end
    end
end

local function game_in_front()
    local w = ffi.C.ofree_GetForegroundWindow()
    if w == nil then return false end
    ffi.C.ofree_GetWindowThreadProcessId(w, pid_out)
    return pid_out[0] == ffi.C.ofree_GetCurrentProcessId()
end

local function stick(v)
    local a = v / 32767
    if math.abs(a) < PAD_DEAD then return 0 end
    return sign(a) * math.min(1, (math.abs(a) - PAD_DEAD) / (1 - PAD_DEAD))
end

-- this frame's input: move (right, up, forward), turn (yaw, pitch), zoom, fast; toggle once a press
local free_was = false
local function free_input()
    local inp = { mx = 0, my = 0, mz = 0, yaw = 0, pitch = 0, zoom = 0, fast = false, toggle = false }
    if not ffi then return inp end
    local pad = get_pad()
    if not game_in_front() then return inp end
    local function key(k) return bit.band(ffi.C.ofree_GetAsyncKeyState(k), 0x8000) ~= 0 end
    local function axis(minus, plus) return (key(plus) and 1 or 0) - (key(minus) and 1 or 0) end
    local K = FREE_KEYS
    inp.mx, inp.my, inp.mz = axis(K.left, K.right), axis(K.down, K.up), axis(K.back, K.fwd)
    inp.yaw, inp.pitch, inp.zoom = axis(K.tl, K.tr), axis(K.td, K.tu), axis(K.zout, K.zin)
    inp.fast = key(K.fast)
    local down = key(FREE_KEY)
    if pad then
        inp.mx, inp.mz = inp.mx + stick(pad.lx), inp.mz + stick(pad.ly)
        inp.yaw, inp.pitch = inp.yaw + stick(pad.rx), inp.pitch + stick(pad.ry)
        inp.my = inp.my + (pad.rt - pad.lt) / 255
        local b = pad.buttons
        inp.zoom = inp.zoom + (bit.band(b, 0x0200) ~= 0 and 1 or 0) - (bit.band(b, 0x0100) ~= 0 and 1 or 0)
        down = down or bit.band(b, FREE_PAD) == FREE_PAD
    end
    inp.toggle = down and not free_was
    free_was = down
    return inp
end

-- the free camera starts where our camera is: eye, the direction it looks (yaw, pitch) and its fov
local function free_start(cam, now)
    local dx, dy, dz = cam[1] - cam[4], cam[2] - cam[5], cam[3] - cam[6]
    return { x = cam[4], y = cam[5], z = cam[6], yaw = math.atan2(dx, dz),
             pitch = math.atan2(dy, math.sqrt(dx * dx + dz * dz)), fov = cam[7], t = now }
end

local function free_camera(f, inp, now, speed, right)
    local dt = math.min(0.1, now - f.t)
    f.t = now
    local turn = FREE_TURN * dt * math.max(0.2, f.fov / 0.8)   -- slower when zoomed in
    f.yaw = f.yaw + right * inp.yaw * turn
    f.pitch = math.max(-1.45, math.min(1.45, f.pitch + inp.pitch * turn))
    f.fov = math.max(0.05, math.min(1.2, f.fov * math.exp(-inp.zoom * FREE_ZOOM * dt)))
    local s = speed * (inp.fast and FREE_FAST or 1) * dt
    local sy, cy = math.sin(f.yaw), math.cos(f.yaw)
    local rx, rz = right * cy, -right * sy                  -- right, level
    f.x = math.max(-80, math.min(80, f.x + (sy * inp.mz + rx * inp.mx) * s))
    f.z = math.max(-70, math.min(70, f.z + (cy * inp.mz + rz * inp.mx) * s))
    f.y = math.max(0.2, math.min(80, f.y + inp.my * s))
    local cp = math.cos(f.pitch)
    return { f.x + cp * sy * 10, f.y + math.sin(f.pitch) * 10, f.z + cp * cy * 10, f.x, f.y, f.z, f.fov }
end

-- the event: fixed for its whole length (the game cuts between several cameras while the line is shown)
local function new_event(ctx, now, why)
    local gx, gy, gz = fat(cam_data + 0x14), fat(cam_data + 0x18), fat(cam_data + 0x1c)
    local ez = fat(cam_data + 0x28)
    local x = line_hooked() or gx
    ev = { t0 = now, x = x, z = math.max(-30, math.min(30, gz)), side = sign(ez - gz),
        tournament = banner_tournament(ctx) }
    log(string.format("offside: VAR camera on (%s) at %.2f, tournament %d, line x %s, look-at x %.2f, game look-at (%.1f, %.1f, %.1f), eye z %.1f",
        why, now, ev.tournament, tostring(line_hooked()), x, gx, gy, gz, ez))
end

local function frame(ctx)
    local now = os.clock()
    local c = cfg()
    local inp = free_input()   -- every frame, so a press counts once
    local vt = u64at(cam_data + 0x58)
    local offside_cam_on = vt ~= last_vt and vt == offside_vt
    last_vt = vt
    track_replay(now)
    -- the drawn wall decides; the offside camera only when the line model could not be hooked
    local obj, n = wall_drawn(now, ev and ev.lead)
    -- The offside replay opens on the offside camera and plays about a second of its own slow run-in
    -- before the wall shows: the lead-up starts with that first frame (the camera comes on right after
    -- the replay logo), so the play never jumps back once it is seen.
    -- (once a replay, right after its player started: the game may cut back to the offside camera later)
    local fresh = replay.t_start and replay.t_start ~= leadup_replay and now - replay.t_start < 0.5
    if not ev and offside_cam_on and fresh and model_data and c.leadup > 0 then
        leadup_replay = replay.t_start
        new_event(ctx, now, "offside replay starting")
        start_leadup(ev, c.leadup)
        if ev.lead then ev.prewall = true else ev = nil end
    end
    if ev and ev.prewall and not ev.lead then
        if obj then
            ev.prewall, ev.t0 = nil, now   -- the wall shows: its timeline starts
        elseif now - ev.t0 > c.leadup + 5 then
            log("offside: no wall after the lead-up")
            ev.prewall = nil
        end
    end
    if not (obj or (ev and ev.prewall) or (not model_data and vt == offside_vt)) then
        if ev then
            log(string.format("offside: VAR camera off after %.2f s", now - ev.t0))
            ev = nil
        end
        freeze_replay(false)
        set_weight(0)
        show_banner(now, -1, 0)
        return
    end
    if not ev then
        new_event(ctx, now, obj and "wall drawn" or "offside camera")
        if not (replay.t_start and replay.t_start == leadup_replay) then   -- not twice a replay
            leadup_replay = replay.t_start
            start_leadup(ev, c.leadup)
        end
    end
    local leading = ev.lead
    if leadup(ev, obj, now, c.leadup) or ev.prewall then   -- the wide view: lead-up and run-in
        ev.from = leadup_camera(ev, c.leadup_frame)
        aim(ev.from)
        set_weight(1)
        return
    end
    if leading then n = 0 end   -- the lead-up just ended: the wall starts growing
    local t = now - ev.t0
    if t < VAR_RAMP then
        ev.x = line_hooked() or ev.x   -- in case the line is set a frame after it shows
    end
    -- the vertical view, held; then on to the offside player once he is found (from then on the
    -- move and the end hold run their full length)
    local player = find_player(ev)
    if player and not ev.move_t0 and t >= c.vertical_hold then ev.move_t0 = t end
    -- preset 1: move to the player view and hold it; preset 2: move to the fly's start, then fly round
    -- him across the wall's plane to its end (looking at him all the way) and hold there
    local fly = c.player_preset >= 2
    local fly_time = fly and c.fly_time or 0
    local hold = (ev.move_t0 or c.vertical_hold) + c.player_move + fly_time + c.end_hold
    if c.debug_freeze > 0 then hold = math.max(hold, c.debug_freeze) end
    if inp.toggle and not ev.free_done then   -- the free camera: on, or off for good
        if ev.free then
            ev.free, ev.free_done = nil, true
            log("offside: free camera off - the replay goes on")
        else
            ev.free = true   -- starts below, from this frame's camera
            log("offside: free camera on - frozen until it is off")
        end
    end
    if ev.free then hold = t + 1 elseif ev.free_done then hold = 0 end
    if obj then
        hold_wall(ev, obj, n, t, hold)
    end
    local cam = vertical_camera(ev.x, ev.side, c)
    local a = 0   -- the view's angle off the wall's plane (the banner shows at the vertical view)
    if ev.move_t0 then
        local tm = t - ev.move_t0
        local deg = c.player_angle
        if fly then
            deg = c.fly_from + (c.fly_to - c.fly_from) * smooth((tm - c.player_move) / fly_time)
        end
        local k = smooth(tm / math.max(0.01, c.player_move))
        local to = player_camera(ev, c, player, deg)
        for i = 1, 7 do cam[i] = cam[i] + (to[i] - cam[i]) * k end
        a = math.rad(deg) * k
    end
    if ev.from then   -- after a lead-up: glide from its wide view (already fully ours)
        local k = smooth(t / LEADUP_GLIDE)
        for i = 1, 7 do cam[i] = ev.from[i] + (cam[i] - ev.from[i]) * k end
    end
    if ev.free == true then ev.free = free_start(cam, now) end
    if ev.free then
        cam = free_camera(ev.free, inp, now, c.free_speed, c.free_invert >= 1 and -1 or 1)
        ev.free_cam = cam
    elseif ev.free_done then
        cam = ev.free_cam   -- stays put while the wall shrinks, then cuts back to the game
    end
    aim(cam)
    set_weight(ev.from and 1 or smooth(t / VAR_RAMP))
    update_banner(ev, now, c.banner >= 1 and t >= VAR_RAMP and not (ev.free or ev.free_done), a, c.banner_time)
end

local function init_zone()
    local addr = memory.search_process(SIGNATURE)
    if not addr then
        log("offside: colour code not found - leaving the offside zone at stock darkness")
        return
    end
    local at = addr + #SIGNATURE
    helper.set_base("zone", at)   -- the overlay's zone_alpha is written here (helper.apply_settings)
    log(string.format("offside: zone alpha %d at %s (v%s)", memory.unpack("u8", memory.read(at, 1)),
        hex(at), m.version))
end

-- hook the line model update's entry: save `this`, count calls, then run the moved prologue
local function init_model_hook()
    local anchor = match_global and memory.search_process(MODEL_ANCHOR)
    if not anchor then
        log("offside: line model update not found - VAR camera follows the offside camera only")
        return
    end
    local entry = addr(anchor) - 0x2a
    local head = memory.read(entry, 14)
    local cave
    if head == MODEL_ENTRY then
        -- the game runs this function every frame of a match: patching it then (a Sider reload) can
        -- catch the game thread inside the bytes being written, so only patch it outside a match
        if match_global and valid_ptr(u64at(match_global)) then
            log("offside: reloaded during a match - restart the game to hook the line model; " ..
                "until then the VAR camera follows the offside camera only")
            return
        end
        cave = addr(memory.allocate_codecave(256))
        model_data = cave + 0x80   -- code is ~0x60 bytes
        memory.write(model_data, string.rep("\0", 0x18))
        memory.write(cave, "\x48\xb8" .. memory.pack("u64", model_data) ..   -- mov rax,data
            "\x48\x89\x08" ..                                              -- mov [rax],rcx    this
            "\xff\x40\x08" ..                                              -- inc dword [rax+8]
            -- the line x, read like the function does later (r10, r11: scratch, not arguments)
            "\x49\xbb" .. memory.pack("u64", match_global) ..               -- mov r11,global
            "\x4d\x8b\x1b" ..                                              -- mov r11,[r11]
            "\x4d\x85\xdb" ..                                              -- test r11,r11
            "\x74\x22" ..                                                  -- jz skip
            "\x4d\x8b\x93\x88\x03\x00\x00" ..                              -- mov r10,[r11+388]  replay manager
            "\x4c\x89\x50\x10" ..                                          -- mov [rax+10],r10
            "\x4d\x8b\x9b\x70\x02\x00\x00" ..                              -- mov r11,[r11+270]  out-of-play info
            "\x4d\x85\xdb" ..                                              -- test r11,r11
            "\x74\x0b" ..                                                  -- jz skip
            "\x45\x8b\x9b\xa8\x15\x00\x00" ..                              -- mov r11d,[r11+15a8] line x
            "\x44\x89\x58\x0c" ..                                          -- mov [rax+c],r11d
            -- skip:
            MODEL_ENTRY ..                                                 -- moved prologue
            jmp_abs(entry + 14))
        memory.write(entry, jmp_abs(cave))
        log(string.format("offside: line model hook at %s, code cave at %s", hex(entry), hex(cave)))
    elseif head:sub(1, 6) == "\xff\x25\x00\x00\x00\x00" then
        cave = u64at(entry + 6)
        model_data = u64at(cave + 2)   -- the cave's first instruction: mov rax,data
        log(string.format("offside: line model hook already in place, code cave at %s", hex(cave)))
    else
        log("offside: line model update differs - VAR camera follows the offside camera only")
    end
end

-- hook the replay player's entry: zero the speed while the flag is set, then run the moved prologue
local function init_speed_hook()
    local entry = memory.search_process(SPEED_ENTRY .. SPEED_AFTER)
    if entry then
        entry = addr(entry)
        if match_global and valid_ptr(u64at(match_global)) then
            log("offside: reloaded during a match - restart the game to hook the replay player; " ..
                "until then the players move on while the wall is held")
            return
        end
        local cave = addr(memory.allocate_codecave(128))
        speed_data = cave + SPEED_DATA   -- code is 0x44 bytes
        memory.write(speed_data, string.rep("\0", 0x10))
        memory.write(cave, "\x48\xb8" .. memory.pack("u64", speed_data) ..   -- mov rax,data
            "\x48\x89\x48\x08" ..                                          -- mov [rax+8],rcx  the player
            "\x80\x38\x00" ..                                              -- cmp byte [rax],0
            "\x74\x0f" ..                                                  -- je normal
            "\x80\x38\x02" ..                                              -- cmp byte [rax],2
            "\x74\x05" ..                                                  -- je force
            "\x0f\x57\xc9" ..                                              -- xorps xmm1,xmm1  speed 0
            "\xeb\x05" ..                                                  -- jmp normal
            -- force:
            "\xf3\x0f\x10\x48\x04" ..                                      -- movss xmm1,[rax+4]
            -- normal:
            SPEED_ENTRY ..                                                 -- moved prologue
            jmp_abs(entry + #SPEED_ENTRY))
        memory.write(entry, jmp_abs(cave) .. string.rep("\x90", #SPEED_ENTRY - 14))
        log(string.format("offside: replay player hook at %s, code cave at %s", hex(entry), hex(cave)))
        return
    end
    -- a reload: find the patched entry by its padding and the code right after it
    local after = memory.search_process(string.rep("\x90", #SPEED_ENTRY - 14) .. SPEED_AFTER)
    local cave = after and u64at(addr(after) - 8)
    if cave and memory.read(cave, 2) == "\x48\xb8" then
        speed_data = u64at(cave + 2)
        memory.write(speed_data, "\0")
        log(string.format("offside: replay player hook already in place, code cave at %s", hex(cave)))
        if speed_data ~= cave + SPEED_DATA then   -- before 3.5: freeze only
            speed_data_v1 = true
            log("offside: replay player hook from an older version - restart the game for the lead-up")
        end
    else
        log("offside: replay player not found - the players move on while the wall is held")
    end
end

local function init_camera(ctx)
    local ctor = memory.search_process(OFFSIDE_CTOR)
    local mg = memory.search_process(MATCH_GLOBAL)
    if not ctor then
        log("offside: CameraOffside not found - VAR camera disabled")
        return
    end
    local vt_at = ctor + #OFFSIDE_CTOR
    offside_vt = addr(vt_at) + 4 + memory.unpack("i32", memory.read(vt_at, 4))
    if mg then
        match_global = addr(mg) + 14 + memory.unpack("i32", memory.read(mg + 10, 4))
    end
    local cave
    local site = memory.search_process(CAM_PREFIX .. CAM_ORIG)
    if site then
        local hook = site + #CAM_PREFIX
        local out = hook + 7 + memory.unpack("i32", memory.read(hook + 3, 4))
        local flag = hook + 17 + memory.unpack("i32", memory.read(hook + 13, 4))
        if memory.read(hook + 7, 6) ~= "\xff\x50\x20\x0f\xb6\x15" then
            log("offside: camera update code differs - VAR camera disabled")
            return
        end
        cave = addr(memory.allocate_codecave(CAM_CODE_MAX + CAM_DATA_SIZE))
        cam_data = cave + CAM_CODE_MAX
        memory.write(cam_data, string.rep("\0", CAM_DATA_SIZE))
        memory.write(cave, build_cam_cave(out, flag, cam_data, hook + 17))
        -- jmp qword [rip]: 14 bytes over lea (7) + call (3) + movzx (7)
        memory.write(hook, jmp_abs(cave) .. "\x90\x90\x90")
        log(string.format("offside: camera hook at %s, output %s, code cave at %s", hex(hook), hex(out), hex(cave)))
    else
        -- a reload: the hook is already in place, reuse its cave
        site = memory.search_process(CAM_PREFIX .. "\xff\x25\x00\x00\x00\x00")
        if not site then
            log("offside: camera update code not found - VAR camera disabled")
            return
        end
        cave = u64at(site + #CAM_PREFIX + 6)
        cam_data = u64at(cave + 0xf)   -- the code's mov rax,data
        memory.write(cam_data, memory.pack("f", 0))
        log(string.format("offside: camera hook already in place, code cave at %s", hex(cave)))
        -- an older version's code: rebuild it (same output, flag and return) when it fits, outside a
        -- match only, as the game runs it every frame of a match
        local code = memory.read(cave, cam_data - cave)
        local tail = code:find("\x0f\xb6\x10\xff\x25\x00\x00\x00\x00", 1, true)
        local new = tail and build_cam_cave(u64at(cave + 2), u64at(cave + tail - 9), cam_data, u64at(cave + tail + 8))
        if new and code:sub(1, #new) ~= new then
            if #new > #code or (match_global and valid_ptr(u64at(match_global))) then
                log("offside: camera hook from an older version - restart the game for a level camera")
            else
                memory.write(cam_data + 4, string.rep("\0", CAM_DATA_SIZE - 4))
                memory.write(cave, new)
                log("offside: camera hook code updated")
            end
        end
    end
    memory.write(cam_data + 0x60, memory.pack("f", 0) .. memory.pack("f", 1) .. memory.pack("f", 0) ..   -- level
        memory.pack("f", NEAR_CLIP))
    cam_out = u64at(cave + 2)   -- the code's mov rdx,out
    log(string.format("offside: CameraOffside vtable %s, match global %s", hex(offside_vt),
        match_global and hex(match_global) or "not found"))
    init_model_hook()
    init_speed_hook()
    m.display_frame = frame   -- kept alive in m (see m.init)
    ctx.register("display_frame", m.display_frame)
end

function m.init(ctx)
    if not ctx.nesalib then
        error("nesalib not found. Install nesalib")
    end
    helper = ctx.nesalib.helper(m.version, "modules\\offside.ini", OVERLAY, SETTINGS_INFO)
    helper.load_ini(ctx)
    helper.set_ini_comment("Offside wall and VAR camera settings")
    init_zone()
    init_camera(ctx)
    helper.apply_settings(ctx, true)   -- zone darkness now; the camera reads the rest each frame
    -- Every handler lives in m: Sider does not keep registered functions alive, and an anonymous one
    -- gets garbage-collected and then called (crash in sider.dll's Lua VM).
    m.overlay_on = helper.overlay_on
    m.key_down = helper.key_down
    m.gamepad_input = helper.gamepad_input
    ctx.register("overlay_on", m.overlay_on)
    ctx.register("key_down", m.key_down)
    ctx.register("gamepad_input", m.gamepad_input)
end

return m
