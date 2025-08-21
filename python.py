from sources import pybye, pymod
import os

"""%
yaml = {}
function yaml.load(filename)
    local f = io.open(filename, "r")
    if not f then error("File not found: " .. filename) end

    local output = {}
    for line in f:lines() do
        if line:match(":") then
            local key, value = line:gmatch("(%w+)%s*:%s*(.+)")()
            if key then output[key] = value end
        end
    end

    return output
end

local config = yaml.load("config.yaml")
local optimize = tonumber(config.age) > 18
print(config.name .. " is " .. (optimize and "optimized" or "not optimized"))

%"""

"""%if optimize then%"""
name = "optimize"

"""%%if true then%%"""
print(f"Hi, I'm {name}")
"""%%end%%"""

print(f"Running on {os.getcwd()}")
"""%else%"""
print("Hi, I'm not optimized")
"""%end%"""

pymod.hi()
pybye.bye()
