import sources.pybye as pybye
import sources.pymod as pymod

# TODO: Try with no preprocessing stuff inside of a file
# FIX: Commenting preprocessing


"""%
print("Preprocessing...")
debug = false
optimized = true
%"""

"""%if debug then%"""
print("Hi, I'm Python")
"""%end%"""

pymod.hi()
pybye.bye()
