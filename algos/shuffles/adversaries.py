@Shuffle("Quicksort Adversary")
def quickSortAdversary(array):
    j = len(array)-len(array) % 2-2
    i = j-1
    while i >= 0:
        array[i].swap(array[j])
        i -= 2
        j -= 1


class GrailSortAdversary:
    def __init__(this):
        this.rotate = sortingVisualizer.getRotationByName("Gries-Mills").indexed

    def push(this, array, a, b, bLen):
        len = b-a
        b1 = b-len % bLen
        len1 = b1-a
        if len1 <= 2*bLen:
            return
        m = bLen
        while 2*m < len:
            m *= 2
        m += a
        if b1-m < bLen:
            this.push(array, a, m, bLen)
        else:
            m = a+b1-m
            rotate = this.rotate
            rotate(array, m-(bLen-2), b1-(bLen-1), b1)
            shift(array, a, m)
            rotate(array, a, m, b1)
            m = a+b1-m
            this.push(array, a, m, bLen)
            this.push(array, m, b, bLen)

    def run(this, array):
        if len(array) <= 16:
            reverse(array, 0, len(array))
        else:
            bLen = 1
            while bLen*bLen < len(array):
                bLen *= 2
            numKeys = (len(array)-1)//bLen+1
            keys = bLen+numKeys
            shuffleRandom(array, 0, len(array))
            UniV_immediateSort(array, 0, keys)
            reverse(array, 0, keys)
            UniV_immediateSort(array, keys, len(array))
            this.push(array, keys, len(array), bLen)


@Shuffle("Grailsort Adversary")
def grailSortAdversary(array):
    GrailSortAdversary().run(array)
