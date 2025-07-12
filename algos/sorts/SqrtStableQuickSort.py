class SqrtStableQuickSort:
    SMALL_SORT = 32

    def __init__(this, rot):
        if rot is None:
            this.rotate = UniV_getUserRotation("Select rotation algorithm", "Helium").indexed
        else:
            this.rotate = sortingVisualizer.getRotationByName(rot).indexed
        this.zeros = None
        this.ones = None
        this.keys = None

    def buildBlocks(this, array, a, b, p):
        this.zeroPtr = 0
        this.onePtr = 0
        last = a
        zeroKey = 0
        oneKey = -1
        keyPtr = 0
        i = a
        while i < b:
            if array[i] <= p:
                this.zeros[this.zeroPtr].write(array[i])
                this.zeroPtr += 1
                if this.zeroPtr == this.bufLen:
                    arrayCopy(this.zeros, 0, array, last, this.bufLen)
                    this.zeroPtr = 0
                    last += this.bufLen
                    this.keys[keyPtr].write(zeroKey)
                    keyPtr += 1
                    zeroKey += 1
            else:
                this.ones[this.onePtr].write(array[i])
                this.onePtr += 1
                if this.onePtr == this.bufLen:
                    arrayCopy(this.ones, 0, array, last, this.bufLen)
                    this.onePtr = 0
                    last += this.bufLen
                    this.keys[keyPtr].write(oneKey)
                    keyPtr += 1
                    oneKey -= 1
            i += 1
        keyPtr -= 1
        maxKey = 0
        r = keyPtr
        i = keyPtr
        while i >= 0:
            if this.keys[i] >= 0:
                maxKey = this.keys[i].readInt()
                break
            i -= 1
        while keyPtr >= 0:
            if this.keys[keyPtr] < 0:
                this.keys[keyPtr].write(maxKey-this.keys[keyPtr].getInt())
            keyPtr -= 1
        return last, a+(zeroKey*this.bufLen), r

    def sortBlocks(this, array, a, k):
        c: int
        i = 0
        while i < k:
            c = 0
            while this.keys[i] != i and c < k:
                blockSwap(array, a+(i*this.bufLen), a +
                          (this.keys[i].getInt()*this.bufLen), this.bufLen)
                this.keys[i].swap(this.keys[this.keys[i].getInt()])
                c += 1
            if c >= k-1:
                break
            i += 1

    def oopPartition(this, array, a, b, p):
        this.zeroPtr = 0
        this.onePtr = 0
        i = a
        while i < b:
            if array[i] <= p:
                this.zeros[this.zeroPtr].write(array[i])
                this.zeroPtr += 1
            else:
                this.ones[this.onePtr].write(array[i])
                this.onePtr += 1
            i += 1
        this.onePtr -= 1
        i = b-1
        while this.onePtr >= 0:
            array[i].write(this.ones[this.onePtr])
            this.onePtr -= 1
            i -= 1
        r = this.zeroPtr
        this.zeroPtr -= 1
        while this.zeroPtr >= 0:
            array[i].write(this.zeros[this.zeroPtr])
            this.zeroPtr -= 1
            i -= 1
        return a+r

    def partition(this, array, a, b, p):
        rotate = this.rotate
        if b-a <= this.bufLen:
            return this.oopPartition(array, a, b, p)
        elif b-a <= this.bufLen*2:
            this.zeroPtr = 0
            this.onePtr = 0
            i = a
            while i < b:
                if array[i] <= p:
                    this.zeros[this.zeroPtr].write(array[i])
                    this.zeroPtr += 1
                else:
                    this.ones[this.onePtr].write(array[i])
                    this.onePtr += 1
                if this.zeroPtr == this.bufLen or this.onePtr == this.bufLen:
                    p0 = a+this.zeroPtr
                    ones = this.onePtr
                    m = p0+ones
                    p1: int
                    arrayCopy(this.zeros, 0, array, a, this.zeroPtr)
                    arrayCopy(this.ones, 0, array, p0, ones)
                    p1 = this.oopPartition(array, m, b, p)
                    rotate(array, p0, m, p1)
                    return p1-ones
                i += 1
        last: int
        m: int
        k: int
        last, m, k = this.buildBlocks(array, a, b, p)
        this.sortBlocks(array, a, k)
        i = 0
        while i < this.onePtr:
            array[last].write(this.ones[i])
            i += 1
            last += 1
        lz = this.zeroPtr
        if lz > 0:
            last -= 1
            i = b-1
            while last >= m:
                array[i].write(array[last])
                i -= 1
                last -= 1
            lz -= 1
            while lz >= 0:
                array[i].write(this.zeros[lz])
                lz -= 1
                i -= 1
        return m+this.zeroPtr

    def getPivot(this, array, a, b):
        sqrt = pow2Sqrt(b-a)
        g = (b-a)//sqrt
        i = a
        j = 0
        while i < b and j < sqrt:
            this.zeros[j].write(array[i])
            i += g
            j += 1
        binaryInsertionSort(this.zeros, 0, sqrt)
        return this.zeros[sqrt//2].copy()

    def quickSorter(this, array, a, b, d):
        while b-a > this.SMALL_SORT:
            if checkSorted(array, a, b):
                return
            if d == 0:
                grailSortGivenAux(array, a, b-a, this.zeros)
                return
            p: int
            l: int
            r: int
            p = this.partition(array, a, b, this.getPivot(array, a, b))
            l = p-a
            r = b-p
            if (l == 0 or r == 0) or (l/r >= 64 or r/l >= 64):
                grailSortGivenAux(array, a, b-a, this.zeros)
                return
            d -= 1
            this.quickSorter(array, a, p, d)
            a = p
        binaryInsertionSort(array, a, b)

    def sort(this, array, a, b):
        sqrt = pow2Sqrt(b-a)
        this.bufLen = sqrt
        this.zeros = sortingVisualizer.createValueArray(sqrt)
        this.ones = sortingVisualizer.createValueArray(sqrt)
        this.keys = sortingVisualizer.createValueArray(((b-a-1)//sqrt)+1)
        sortingVisualizer.setNonOrigAux([this.keys])
        this.quickSorter(array, a, b, 2*math.log2(b-a))


@Sort("Quick Sorts", "Sqrt Stable QuickSort", "Sqrt Stable Quick")
def sqrtStableQuickSortRun(array):
    SqrtStableQuickSort(None).sort(array, 0, len(array))
