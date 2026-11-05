-- Aspect ratio fix for SP Football Life 2027 (Sider module)
-- Based on lua by Durandil67.
-- Auto-installed by the Lutris installer when the screen ratio != 16:9.
-- See: lutris/sp-football-life-2027.yaml ("Setting up aspect ratio fix...")
-- Note: this patch only applies during matches, not in menus.

local m = { version = "1.0" }

function m.init(ctx)
    local addr = memory.search_process("\x80\x7B\x7C\x00\x0F\x28")
    if addr then
        log(string.format("found magic string at: %s", memory.hex(addr)))
        memory.write(addr, "\x80\x7B\x7C\x01\x0F\x28")
    else
        error("unable to find magic string")
    end
end

return m
