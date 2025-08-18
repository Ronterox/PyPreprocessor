from sources import pybye, pymod
import os

"""%
optimize = true
local f = io.open("python.py", "r")
if not f then error("File not found") end

for line in f:lines() do
    if line:match("=") then
        local name, value = line:gmatch("(%w+)%s*=%s*(.+)")()
        print(name, value)
    end
end
%"""

name = "optimize"
print(f"Hi, I'm {name}")
print(f"Running on {os.getcwd()}")

pymod.hi()
pybye.bye()
