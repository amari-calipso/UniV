class PoplarHeapSort:
    def hyperfloor(this, n):
        return 2**int(math.log2(n))

    def uncheckedInsertionSort(this, array, a, b):
        i = a+1
        while i != b:
            sift = i
            sift1 = i-1
            if array[sift] < array[sift1]:
                tmp = array[sift].copy()
                while True:
                    array[sift].write(array[sift1].noMark())
                    sift -= 1
                    sift1 -= 1
                    if not (sift != a and tmp < array[sift1]):
                        break
                array[sift].write(tmp)
            i += 1

    def insertionSort(this, array, a, b):
        if a == b:
            return
        this.uncheckedInsertionSort(array, a, b)

    def sift(this, array, a, l):
        if l < 2:
            return
        r = a+l-1
        c1 = r-1
        c2 = a+(l//2-1)
        while True:
            maxR = r
            if array[maxR] < array[c1]:
                maxR = c1
            if array[maxR] < array[c2]:
                maxR = c2
            if maxR == r:
                return
            array[r].swap(array[maxR])
            l //= 2
            if l < 2:
                return
            r = maxR
            c1 = r-1
            c2 = maxR-(l-l//2)

    def popHeapWithSize(this, array, a, b, l):
        poplarSize = this.hyperfloor(l+1)-1
        lR = b-1
        bigger = lR
        biggerSize = poplarSize
        it = a
        while True:
            r = it+poplarSize-1
            if r == lR:
                break
            if array[bigger] < array[r]:
                bigger = r
                biggerSize = poplarSize
            it = r+1
            l -= poplarSize
            poplarSize = this.hyperfloor(l+1)-1
        if bigger != lR:
            array[bigger].swap(array[lR])
            this.sift(array, bigger-(biggerSize-1), biggerSize)

    def makeHeap(this, array, a, b):
        l = b-a
        if l < 2:
            return
        smallPoplarSize = 15
        if l <= smallPoplarSize:
            this.uncheckedInsertionSort(array, a, b)
            return
        poplarLevel = 1
        it = a
        next = it+smallPoplarSize
        while True:
            this.uncheckedInsertionSort(array, it, next)
            poplarSize = smallPoplarSize
            i = (poplarLevel & (-poplarLevel)) >> 1
            while i != 0:
                it -= poplarSize
                poplarSize = 2*poplarSize+1
                this.sift(array, it, poplarSize)
                next += 1
                i >>= 1
            if b-next <= smallPoplarSize:
                this.insertionSort(array, next, b)
                return
            it = next
            next += smallPoplarSize
            poplarLevel += 1

    def sortHeap(this, array, a, b):
        l = b-a
        if l < 2:
            return
        while True:
            this.popHeapWithSize(array, a, b, l)
            b -= 1
            l -= 1
            if not (l > 1):
                break

    def sort(this, array, a, b):
        this.makeHeap(array, a, b)
        this.sortHeap(array, a, b)


@Sort("Tree Sorts", "Poplar Heap Sort", "Poplar Heap")
def poplarHeapSortRun(array):
    PoplarHeapSort().sort(array, 0, len(array))
