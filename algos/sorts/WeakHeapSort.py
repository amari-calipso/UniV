class WeakHeapSort:
    def __init__(this, array, length):
        this.array = array
        this.length = length
        this.bits = sortingVisualizer.createValueArray((this.length+7)//8)
        sortingVisualizer.setNonOrigAux([this.bits])

    def getBitwiseFlag(this, x):
        return (this.bits[x >> 3] >> (x & 7)) & 1

    def toggleBitwiseFlag(this, x):
        flag = this.bits[x >> 3].copy()
        flag ^= 1 << (x & 7)
        this.bits[x >> 3].write(flag)

    def merge(this, i, j):
        if this.array[i] < this.array[j]:
            this.toggleBitwiseFlag(j)
            this.array[i].swap(this.array[j])

    def sort(this):
        n = this.length
        i: int
        j: int
        x: int
        y: int
        Gparent: int
        i = n-1
        while i > 0:
            j = i
            while (j & 1) == this.getBitwiseFlag(j >> 1):
                j >>= 1
            Gparent = j >> 1
            this.merge(Gparent, i)
            i -= 1
        i = n-1
        while i >= 2:
            this.array[0].swap(this.array[i])
            x = 1
            y = 2*x+this.getBitwiseFlag(x)
            while y < i:
                y = 2*x+this.getBitwiseFlag(x)
                if y >= i:
                    break
                x = y
            while x > 0:
                this.merge(0, x)
                x >>= 1
            i -= 1
        this.array[0].swap(this.array[1])


@Sort("Tree Sorts", "Weak Heap Sort", "Weak Heap Sort")
def weakHeapSortRun(array):
    WeakHeapSort(array, len(array)).sort()
