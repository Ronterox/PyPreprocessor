"""%print("Slow loading pymod...")%"""

import pybye as bye


def hi():
    print("Hi I'm a module")
    """%if not optimized then%"""
    for i in range(1000):
        if i % 100 == 0:
            print("Costly operation: " + str(i))
    """%end%"""


hi()
bye.bye()
