from sources import pybye, pymod
import os

"""%optimize = true%"""

print("Hi, I'm Python")
print(f"Running on {os.getcwd()}")

pymod.hi()
pybye.bye()
