"""%
print("Slow loading pymod...")
optimize = true
%"""

import pybye as bye
import os


def hi():
    print("Hi I'm a module")
    """%if not optimize then%"""
    for i in range(1000):
        if i % 100 == 0:
            print("Costly operation: " + str(i))
    """%end%"""


print(f"Running on {os.getcwd()}")
hi()
bye.bye()
