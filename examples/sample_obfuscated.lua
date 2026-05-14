-- sample_obfuscated.lua
-- An obfuscated Lua 5.1 script implementing string decoding, control flow flattening, and an opaque predicate.

local _STRS = {
    [1] = "\98\79\70\70\69", -- Hello
    [2] = "\6\10\110\79\69\72\76\95\89\73\75\94\79\78\10\125\69\88\70\78\11", -- , Deobfuscated World!
    [3] = "\90\88\67\68\94", -- print
    [4] = "\89\94\88\67\68\77", -- string
    [5] = "\94\75\72\70\79", -- table
    [6] = "\73\69\68\73\75\94", -- concat
    [7] = "\70\69\73\75\70", -- local
    [8] = "\77\88\79\79\94\67\68\77", -- greeting
}

local function decode(s, k)
    local r = {}
    for i = 1, #s do
        r[i] = string.char(bit32.bxor(s:byte(i), k))
    end
    return table.concat(r)
end

local _STATE = 3
local _greeting
local _suffix
local _msg
local _p

while true do
    if (_STATE % 1 == 0) then
        if _STATE == 1 then
            _greeting = decode(_STRS[1], 42)
            _STATE = 5
        elseif _STATE == 2 then
            _p(_msg)
            _STATE = 6
        elseif _STATE == 3 then
            _STATE = 1
        elseif _STATE == 4 then
            _msg = _greeting .. _suffix
            _p = _G[decode(_STRS[3], 42)]
            _STATE = 2
        elseif _STATE == 5 then
            _suffix = decode(_STRS[2], 42)
            _STATE = 4
        elseif _STATE == 6 then
            break
        end
    else
        -- Dead code inside opaque predicate failure
        _G[decode(_STRS[3], 42)]("This shouldn't happen")
    end
end
