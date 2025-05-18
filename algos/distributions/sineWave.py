import math

@Distribution("Sine Wave")
def sineWaveDist(array):
    length = len(array)
    n = length-1
    c = 2*math.pi/n
    for i in range(length):
        array[i] = int(n*(math.sin(c*i)+1)/2)
