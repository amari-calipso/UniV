@Distribution("Quintic")
def quinticDist(array):
    length = len(array)
    mid = (length-1)/2
    for i in range(length):
        array[i] = int((((i-mid)**5)/(mid**4))+mid)
