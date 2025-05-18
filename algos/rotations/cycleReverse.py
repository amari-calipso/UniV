@Rotation("Cycle Reverse", RotationMode.INDEXED)
def cycleReverseRotate(array, a, m, e):
    lenA = m-a
    lenB = e-m
    if lenA < 1 or lenB < 1:
        return
    b = m-1
    c = m
    d = e-1
    swap: Value
    while a < b and c < d:
        swap = array[b].read()
        array[b].write(array[a])
        b -= 1
        array[a].write(array[c])
        a += 1
        array[c].write(array[d])
        c += 1
        array[d].write(swap)
        d -= 1
    while a < b:
        swap = array[b].read()
        array[b].write(array[a])
        b -= 1
        array[a].write(array[d])
        a += 1
        array[d].write(swap)
        d -= 1
    while c < d:
        swap = array[c].read()
        array[c].write(array[d])
        c += 1
        array[d].write(array[a])
        d -= 1
        array[a].write(swap)
        a += 1
    if a < d:
        reverse(array, a, d+1)
