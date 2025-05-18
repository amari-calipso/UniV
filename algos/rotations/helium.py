@Rotation("Helium", RotationMode.INDEXED)
def heliumRotate(array, a, m, b):
    rl = b-m
    ll = m-a
    while ll > 1 and rl > 1:
        if rl < ll:
            blockSwap(array, a, m, rl)
            a += rl
            ll -= rl
        else:
            b -= ll
            rl -= ll
            backwardBlockSwap(array, a, b, ll)
    if rl == 1:
        insertToLeft(array, m, a)
    elif ll == 1:
        insertToRight(array, a, b-1)
