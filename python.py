import sources.pybye as pybye
import sources.pymod as pymod

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
