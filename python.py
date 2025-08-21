from sources import pybye, pymod
import os

"""%
optimize = false
local f = io.open("config.yaml", "r")
if not f then error("File not found") end

for line in f:lines() do
    if line:match("=") then
        local name, value = line:gmatch("(%w+)%s*=%s*(.+)")()
        print(name, value)
    end
end
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
