local m = { version = "2.4" }
local root, sider_dir, active, selected, base_path, status
local clumps_active, clumps_selected
local served = {}
local grass_active,grass_selected
local visibility_active,visibility_selected

local crew, config, repair_selected, repair_active
local bindings, binding_names = {}, {}
local defaults={ key_brown_less="MINUS",key_brown_more="PLUS",key_marks_less="LBRACKET",key_marks_more="RBRACKET",key_clumps="C",key_repair_toggle="H",key_repair_now="G",key_global_grass="T",key_visibility_less="F8",key_visibility_more="F9" }
local named={MINUS=0xBD,PLUS=0xBB,LBRACKET=0xDB,RBRACKET=0xDD,NUMPLUS=0x6B,NUMMINUS=0x6D,SPACE=0x20,HOME=0x24,END=0x23,PAGEUP=0x21,PAGEDOWN=0x22,INSERT=0x2D,DELETE=0x2E}
local function keycode(value)
    value=value:upper()
    if value=="NONE" then return false end
    if named[value] then return named[value] end
    if value:match("^[A-Z0-9]$") then return value:byte() end
    local f=tonumber(value:match("^F(%d+)$"))
    if f and f>=1 and f<=12 then return 0x6F+f end
    local number=tonumber(value)
    if number and number>=1 and number<=255 and number==math.floor(number) then return number end
    return nil
end
local function setup_keys(values)
    local used={}
    for name,default in pairs(defaults) do
        local text=(values[name] or default):upper()
        local key=keycode(text)
        assert(key~=nil,"Invalid binding "..name.." = "..text)
        if key then assert(not used[key],"Duplicate binding: "..name.." and "..tostring(used[key]));used[key]=name end
        bindings[name]=key;binding_names[name]=text
    end
end
local function extra_settings()
    local text="global_grass_enabled = "..(grass_selected and "1" or "0").."\r\nmarks_visibility = "..visibility_selected.."\r\nrepair_enabled = "..(repair_selected and "1" or "0").."\r\n"
    for _,name in ipairs({"repair_percent","repair_mode","repair_cell","repair_delay","repair_duration"}) do
        text=text..name.." = "..config[name].."\r\n"
    end
    for _,name in ipairs({"key_brown_less","key_brown_more","key_marks_less","key_marks_more","key_clumps","key_repair_toggle","key_repair_now","key_visibility_less","key_visibility_more","key_global_grass"}) do
        text=text..name.." = "..binding_names[name].."\r\n"
    end
    return text
end

local marks_active, marks_selected
local targets = { DrawPitchEffect = { [2] = true }, pes3ddf_turf3d_2020 = { [1] = true },
    pes3ddf_pitch_2020 = { [2] = true }, pes3ddf_pitch_2020_en = { [2] = true } }
local function read(path)
    local f = io.open(path, "rb")
    if not f then return nil end
    local bytes = f:read("*a"); f:close(); return bytes
end
local function write(path, bytes)
    local f, err = io.open(path, "wb")
    if not f then return false, err end
    local ok, message = f:write(bytes); local closed, close_error = f:close()
    return ok ~= nil and closed ~= nil, message or close_error
end
local function u32(s, p)
    local a,b,c,d = s:byte(p,p+3); assert(d, "Truncated FSOP")
    return a + b*256 + c*65536 + d*16777216
end
local function pack32(n)
    return string.char(n%256,math.floor(n/256)%256,math.floor(n/65536)%256,math.floor(n/16777216)%256)
end
local function parse(bytes)
    local rows, p, names = {}, 1, {}
    while p <= #bytes do
        local n=bytes:byte(p); p=p+1; assert(n and n>0 and p+n-1<=#bytes,"Bad FSOP name")
        local label=bytes:sub(p,p+n-1); p=p+n
        local name=label:gsub("%z+$", ""); assert(not names[name], "Duplicate shader name"); names[name]=true
        local row={name=name,label=label,stages={}}
        for stage=1,2 do
            local size=u32(bytes,p); p=p+4; assert(size>0 and p+size-1<=#bytes,"Bad FSOP length")
            row.stages[stage]=bytes:sub(p,p+size-1); p=p+size
        end
        rows[#rows+1]=row
    end
    assert(#rows==244,"Expected PES21 shader package with 244 pairs")
    return rows
end
local function merge(base, preset, writer, visibility_shaders)
    local replacements={}
    for _,row in ipairs(parse(preset)) do replacements[row.name]=row end
    local result, count={},0
    for _,row in ipairs(parse(base)) do
        result[#result+1]=string.char(#row.label)..row.label
        for stage=1,2 do
            local bytes=row.stages[stage]
            if targets[row.name] and targets[row.name][stage] then
                assert(replacements[row.name],"Missing turf shader")
                bytes=replacements[row.name].stages[stage]
                if row.name=="DrawPitchEffect" and stage==2 then bytes=writer end
                if visibility_shaders[row.name] and stage==2 then bytes=visibility_shaders[row.name] end
                count=count+1
            end
            result[#result+1]=pack32(#bytes)..bytes
        end
    end
    assert(count==4,"Could not locate all four turf stages")
    return table.concat(result)
end
local function read_settings()
    local values={}
    local text=read(root.."settings.ini") or ""
    -- A whitespace pattern over the whole file can consume a newline after '='.
    -- Parse each line separately. Last duplicate key wins, comments are ignored.
    for line in (text.."\n"):gmatch("([^\r\n]*)[\r\n]") do
        line=line:gsub("^%s+",""):gsub("%s+$","")
        if line~="" and line:sub(1,1)~=";" and line:sub(1,1)~="#" then
            local key,value=line:match("^([%w_]+)%s*=%s*(.*)$")
            if key then values[key:lower()]=value:gsub("%s+$","") end
        end
    end
    return values
end
local function settings()
    local values=read_settings()
    local percent=tonumber(values.percent) or 30
    percent=math.max(0,math.min(100,math.floor(percent/10+.5)*10))
    local marks=tonumber(values.marks_strength) or 100
    marks=math.max(0,math.min(100,math.floor(marks/10+.5)*10))
    return percent,values.base_fsop or "",values.clumps_enabled~="0",marks
end
local function save()
    local text="; Turf Evolution v2.4 - changes apply after restarting PES and Sider\r\npercent = "..selected.."\r\nbase_fsop = "..base_path.."\r\nclumps_enabled = "..(clumps_selected and "1" or "0").."\r\nmarks_strength = "..marks_selected.."\r\n"
    local ok,err=write(root.."settings.ini",text..extra_settings())
    status=ok and "Saved: restart PES and Sider to apply" or "Cannot save settings (folder permissions)"
    if not ok then log("[turf-evolution] Save failed: "..tostring(err)) end
end
function m.key_down(ctx,key)
    if key==bindings.key_brown_less then selected=math.max(0,selected-10);save()
    elseif key==bindings.key_brown_more then selected=math.min(100,selected+10);save()
    elseif key==bindings.key_clumps then clumps_selected=not clumps_selected;save()
    elseif key==bindings.key_marks_less then marks_selected=math.max(0,marks_selected-10);save()
    elseif key==bindings.key_marks_more then marks_selected=math.min(100,marks_selected+10);save()
    elseif key==bindings.key_repair_toggle then repair_selected=not repair_selected;save()
    elseif key==bindings.key_repair_now and crew then crew.key_down(ctx,0x47)
    elseif key==bindings.key_visibility_less then visibility_selected=math.max(0,visibility_selected-10);save()
    elseif key==bindings.key_visibility_more then visibility_selected=math.min(100,visibility_selected+10);save()
    elseif key==bindings.key_global_grass then grass_selected=not grass_selected;save() end
end
local function wanted(filename)
    return filename:lower():gsub("/","\\"):gsub("^\\+","")=="shaders\\dx11\\pesshaders_dx11.fsop"
end
local function clump_file(filename)
    if not clumps_active then return nil end
    local f=filename:lower():gsub("/","\\")
    local mode,ext=f:match("asset\\model\\bg\\st%d%d%d\\#win\\st%d%d%d_([dn][fr])%.(fpkd?)$")
    if mode then return "clumps\\Asset\\model\\bg\\st000\\#Win\\st000_"..mode.."."..ext end
    local tex=f:match("asset\\model\\bg\\st%d%d%d\\sourceimages\\tga\\#windx11\\([^\\]+%.ftex)$")
    if tex=="cm000_pape001_bsm_alp.ftex" or tex=="cm000_pape001_nrm.ftex" or tex=="cm000_pape001_srm.ftex" or tex=="cm000_black_mtl.ftex" then
        return "clumps\\Asset\\model\\bg\\st000\\sourceimages\\tga\\#windx11\\"..tex
    end
end
local function global_grass_file(filename)
    if not grass_active then return nil end
    local f=filename:lower():gsub(string.char(92),"/"):gsub("^/+","")
    if f=="asset/model/bg/common/turf3d/#win/turf3dtexlodu4.fmdl" then return "turf3dtexlodu4.fmdl" end
    if f=="asset/model/bg/common/turf3d/sourceimages/#windx11/grassfin_bsm_alp.ftex" then return "grassfin_bsm_alp.ftex" end
end
function m.key(ctx,filename)
    local grass=global_grass_file(filename)
    if grass then return "turf-evolution-global-grass:"..grass end
    if wanted(filename) then return "turf-evolution-v1.8:"..active..":"..marks_active end
    local file=clump_file(filename)
    if file then return "turf-evolution-clump-v1:"..filename end
end
function m.path(ctx,filename,key)
    local grass=global_grass_file(filename)
    if grass then
        if not served[grass] then log("[turf-evolution] Global grass: "..grass);served[grass]=true end
        return root.."global-grass/"..grass
    end
    if wanted(filename) then
        log("[turf-evolution] Serving active="..active.."%, saved="..selected.."%")
        return root.."active.fsop"
    end
    local file=clump_file(filename)
    if file then
        if not served[file] then log("[turf-evolution] Static clumps: "..filename);served[file]=true end
        return root..file
    end
end
local function create_crew(options)
-- pitch_crew.lua v1.3 — halftime "grounds crew" trigger for pitch_crew.addon64
-- At the first-half -> second-half transition (match.stats().period 1 -> 2) it
-- writes content\pitch-crew\repair.trigger ("defer"). The ReShade add-on (v5)
-- waits DELAY seconds, then repairs (all at once if DURATION = 0, otherwise tile
-- by tile over DURATION seconds). When the second-half clock starts running it
-- writes repair.force, so anything still pending is done at once.
-- All tunables below travel in the trigger file: edit + Shift-R, no add-on rebuild.
-- [G] on the overlay page repairs immediately (test).
-- Overlay page: [G] = fire a repair now (test), [M] = toggle patch/fade.
-- Needs match-stats.enabled = 1 in sider.ini.

local m = { version = "1.3" }

-- ---------------- tunables ----------------
local FRACTION   = tonumber(options.repair_percent)/100     -- share of wear the crew fixes (0..1)
local MODE       = options.repair_mode  -- "patch" = reset 75% of small cells; "fade" = every mark fades 75%
local CELL       = tonumber(options.repair_cell)        -- patch cell size in texels (~2.5 cm per texel)
local DELAY      = tonumber(options.repair_delay)      -- seconds after the half-time whistle before the crew acts (aim for the camera cut)
local DURATION   = tonumber(options.repair_duration)        -- 0 = all at once; > 0 = gradual over this many seconds
local POLL       = 5        -- frames between match.stats() reads (keeps the DELAY start accurate)
-- ------------------------------------------

local trigger_path, force_path
local forced, saw_reset = false, false
local last_period, fired, frame = nil, false, 0
local status = "waiting for match"

local function fire(reason, when)
    local f, err = io.open(trigger_path, "wb")
    if not f then
        status = "cannot write trigger: " .. tostring(err)
        log("[pitch_crew] trigger write FAILED: " .. tostring(err))
        return
    end
    f:write(string.format("%.2f %s %d %s %.2f %.2f\n", FRACTION, MODE, CELL, when, DELAY, DURATION)); f:close()
    status = string.format("repair requested (%s) %.0f%% %s", reason, FRACTION * 100, MODE)
    log("[pitch_crew] trigger written (" .. reason .. "): " .. trigger_path)
end

function m.display_frame(ctx)
    frame = frame + 1
    if frame % POLL ~= 0 then return end
    if not match or not match.stats then return end
    local s = match.stats()
    if not s then return end
    local p = s.period
    if p ~= last_period then
        log(string.format("[pitch_crew] period %s -> %s (clock %02d:%02d)", tostring(last_period), tostring(p), s.clock_minutes or -1, s.clock_seconds or -1))
        if last_period == 1 and p == 2 and not fired then
            fired = true
            forced, saw_reset = false, false
            fire("half-time", "defer")
        end
        -- v1.1: no reset on period drops (period is not monotonic in extra time);
        -- re-arming happens only in set_teams (new match).
        last_period = p
    end
    -- fallback: second half is running -> force a still-pending repair.
    -- Requires having seen the clock at 45:00 in period 2 first (so a stale
    -- first-half added-time reading can never fire it on screen).
    local secs = s.clock_minutes and (s.clock_minutes * 60 + (s.clock_seconds or 0)) or nil
    if fired and p == 2 and secs == 45 * 60 then saw_reset = true end
    if fired and saw_reset and not forced and p == 2 and secs and secs >= 45 * 60 + 2 then
        forced = true
        local f = io.open(force_path, "wb")
        if f then f:write("force\n"); f:close() end
        log(string.format("[pitch_crew] second half running (clock %02d:%02d): force signal written", s.clock_minutes, s.clock_seconds or 0))
    end
end

function m.set_teams(ctx)
    last_period, fired, forced, saw_reset = nil, false, false, false
    status = "new match: armed"
end

function m.key_down(ctx, vkey)
    if vkey == 0x47 then fire("manual G", "now")              -- G
    elseif vkey == 0x4D then MODE = (MODE == "patch") and "fade" or "patch"; status = "mode -> " .. MODE end  -- M
end

function m.overlay_on(ctx)
    return string.format("PITCH CREW v%s | half-time fix %.0f%% (%s, cell %d) %.1fs after whistle, %s | period %s | %s\n[G] repair now (test)  [M] toggle patch/fade",
        m.version, FRACTION * 100, MODE, CELL, DELAY, DURATION > 0 and string.format("over %.1fs", DURATION) or "all at once", tostring(last_period), status)
end

function m.init(ctx)
    trigger_path = ctx.sider_dir .. "content\\pitch-crew\\repair.trigger"
    force_path   = ctx.sider_dir .. "content\\pitch-crew\\repair.force"
    if not match or not match.stats then log("[pitch_crew] WARNING: match.stats() unavailable - set match-stats.enabled = 1") end
    ctx.register("display_frame", m.display_frame)
    ctx.register("set_teams", m.set_teams)


    log("[pitch_crew] v" .. m.version .. " ready; trigger: " .. trigger_path)
end

return m

end
function m.overlay(ctx)
    local text=string.format("TURF EVOLUTION v2.4\nBrown: %d%% active / %d%% saved [%s/%s]\nMarks: %d%% active / %d%% saved [%s/%s]\nStatic clumps: %s active / %s saved [%s]\nHalftime repair: %s active / %s saved [%s]\n%s",active,selected,binding_names.key_brown_less,binding_names.key_brown_more,marks_active,marks_selected,binding_names.key_marks_less,binding_names.key_marks_more,clumps_active and "ON" or "OFF",clumps_selected and "ON" or "OFF",binding_names.key_clumps,repair_active and "ON" or "OFF",repair_selected and "ON" or "OFF",binding_names.key_repair_toggle,status)
    text=text..string.format("\nGround mark contrast: %d%% active / %d%% saved [%s/%s]",visibility_active,visibility_selected,binding_names.key_visibility_less,binding_names.key_visibility_more)
    text=text..string.format("\nGlobal 3D grass: %s active / %s saved [%s]",grass_active and "ON" or "OFF",grass_selected and "ON" or "OFF",binding_names.key_global_grass)
    if crew then text=text.."\nPitchCrew: "..config.repair_percent.."% repair | test ["..binding_names.key_repair_now.."] | requires ReShade add-on" end
    return text
end
function m.init(ctx)
    sider_dir=ctx.sider_dir;root=sider_dir.."content\\turf-evolution\\"
    selected,base_path,clumps_selected,marks_selected=settings();active=selected;clumps_active=clumps_selected;marks_active=marks_selected
    config=read_settings();setup_keys(config)
    grass_selected=config.global_grass_enabled~="0";grass_active=grass_selected
    visibility_selected=math.max(0,math.min(100,math.floor((tonumber(config.marks_visibility) or 100)/10+.5)*10));visibility_active=visibility_selected
    repair_selected=config.repair_enabled=="1";repair_active=repair_selected
    config.repair_percent=tostring(math.max(0,math.min(100,tonumber(config.repair_percent) or 75)))
    config.repair_mode=config.repair_mode=="fade" and "fade" or "patch"
    config.repair_cell=tostring(math.max(1,math.min(128,math.floor(tonumber(config.repair_cell) or 8))))
    config.repair_delay=tostring(math.max(0,math.min(60,tonumber(config.repair_delay) or 5)))
    config.repair_duration=tostring(math.max(0,math.min(300,tonumber(config.repair_duration) or 0)))
    local preset=assert(read(root.."presets\\"..active..".fsop"),"Missing Turf Evolution preset")
    local writer=assert(read(root.."marks\\"..marks_active..".bin"),"Missing marks-strength shader")
    local visibility_shaders={}
    for _,name in ipairs({"pes3ddf_pitch_2020","pes3ddf_pitch_2020_en"}) do
        visibility_shaders[name]=assert(read(root.."visibility\\"..active.."\\"..visibility_active.."_"..name..".bin"),"Missing visibility shader")
    end
    local bytes=merge(preset,preset,writer,visibility_shaders)
    if base_path~="" then
        local path=base_path
        if not path:match("^%a:[/\\]") and not path:match("^\\\\") then path=sider_dir..path end
        bytes=merge(assert(read(path),"Cannot read base_fsop: "..path),preset,writer,visibility_shaders)
        log("[turf-evolution] Preserved custom base; replaced only 4 grass stages")
    else parse(preset) end
    local ok,err=write(root.."active.fsop",bytes);assert(ok,"Cannot write active.fsop: "..tostring(err))
    status="Restart PES and Sider after changing saved settings"
    ctx.register("livecpk_make_key",m.key);ctx.register("livecpk_get_filepath",m.path)
    ctx.register("overlay_on",m.overlay);ctx.register("key_down",m.key_down)
    if repair_active then crew=create_crew(config);crew.init(ctx) end
    log("[turf-evolution] Initialized brown="..active.."%, marks="..marks_active.."%")
end
return m
