@Distribution("Quadratic")
def quadraticDist(array):
    length = len(array)
    for i in range(length):
        array[i] = (i**2)//length
